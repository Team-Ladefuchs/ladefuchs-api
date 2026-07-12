use super::{
    api::client::{EcoMovementClient, ResponseData, stream_all_data},
    db::Table,
};
use crate::slack::SlackClient;
use crate::slack::{self, TextMessage};
use crate::{eco_movement::api::client::Endpoint, slack::Slack};
use crate::{
    eco_movement::db::{self},
    state::State,
};
use async_trait::async_trait;
use chrono::Utc;
use db::truncate;
use serde::de::DeserializeOwned;
use sqlx::Acquire;
use sqlx::PgConnection;
use std::{any, fmt::Debug};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

use futures_util::{pin_mut, stream::StreamExt};

pub async fn start_import_task(scheduler: &JobScheduler, state: State) -> Result<(), eyre::Error> {
    if state.config.import_on_start {
        run_import_now(&state);
    }

    scheduler
        .add(Job::new_async(
            state.config.cron_schedule.clone(),
            move |uuid: uuid::Uuid, mut lock| {
                Box::pin({
                    let state_value = state.clone();
                    async move {
                        tracing::info!(timestamp = %Utc::now().to_rfc3339(), "trigger import");
                        if let Err(error) = run_import(state_value.clone()).await {
                         if let Some(slack) = &state_value.slack {
							let message_prefix = "Beim Importieren der Daten aus der Eco-Movement-API ist ein Fehler aufgetreten. Deshalb wurde der Import abgebrochen und wird beim nächsten Mal erneut versucht.";
							slack
								.send_error_message(format!("{message_prefix}\nFür Dominic -> [{error}]"))
								.await;
						}
                            tracing::error!(?error, "error during import task")
                        }

                        let next_tick = lock.next_tick_for_job(uuid).await;
                        match next_tick {
                            Ok(Some(ts)) => info!("Next time job is {:?}", ts),
                            _ => error!("Could not get next tick for 7s job"),
                        }
                    }
                })
            },
        )?)
        .await?;

    Ok(())
}

pub fn run_import_now(state: &State) {
    let state = state.clone();
    tokio::task::spawn(async move {
        if let Err(error) = run_import(state).await {
            tracing::error!(?error, "error during run import");
        }
    });
}

pub async fn run_import(state: State) -> Result<(), eyre::Error> {
    let Some(_lock) = state.lock() else {
        tracing::info!("Skipped import because another import is in progress");
        return Ok(());
    };

    let start_time = tokio::time::Instant::now();
    info!("Starting data import");

    let mut connection = state.database_pool.acquire().await?;
    let eco_api = &state.eco_movement_api;

    let max_request_pages = state.config.max_request_pages;

    info!("Importing locations");
    import(
        &mut connection,
        location::LocationImport { eco_api },
        max_request_pages,
    )
    .await?;

    info!("Importing prices");
    import(
        &mut connection,
        price::PriceImport { eco_api },
        max_request_pages,
    )
    .await?;

    info!("Importing connector price data");
    import(
        &mut connection,
        connector_price::ConnectorPriceImport { eco_api },
        max_request_pages,
    )
    .await?;

    let mut transaction = connection.begin().await?;

    info!("Importing operator data");
    operator::import(&mut transaction).await?;

    info!("Importing tariff data");
    tariff::import(&mut transaction).await?;

    info!("import price data");
    let price_result = price::import(&mut transaction).await?;
    info!(price_result);

    if price_result < 200 {
        transaction.rollback().await?;
        let slack = &state.slack;
        slack
            .send_message(TextMessage {
                emoji: Some(slack::Emoji::Down),
                text: String::from(
                    "Beim importieren haben wir keine Preise bekommen von der Eco-Movement Api. Wir werden ganz klar unten gehalten!",
                ),
                markdown: false,
            })
            .await;
        return Ok(());
    }

    let slack = &state.slack;
    let max_standard_operator = 2;
    if send_disabled_operators_info(&mut transaction, slack).await? > max_standard_operator {
        warn!("More stand {max_standard_operator} operator without an price. Abort import");
        transaction.rollback().await?;

        slack
            .send_warning_message(format!("Der Preisimport wurde abgebrochen, weil mehr als {max_standard_operator} Standard-Operatoren keine Preise haben. Bestimmt irgendwas mit unte halten"))
            .await;

        return Ok(());
    }

    info!("import dynamic prices");
    dynamic_price::import(&mut transaction, slack).await?;

    transaction.commit().await?;

    log_duration(start_time);

    Ok(())
}

async fn send_disabled_operators_info(
    connection: &mut PgConnection,
    slack: &Option<Slack>,
) -> Result<usize, eyre::Error> {
    let disabled_operators = db::operator::get_standard_with_no_prices(connection).await?;

    if disabled_operators.is_empty() {
        return Ok(0);
    }

    let disabled_operators_names = disabled_operators.join("\n");
    info!("operator with no prices" = disabled_operators_names);

    slack
        .send_warning_message(format!(
            "Zu diesen Standard CPOs haben wir keine Preise erhalten:\n\n{}",
            &disabled_operators_names,
        ))
        .await;
    Ok(disabled_operators.len())
}

fn log_duration(start_time: tokio::time::Instant) {
    let duration = start_time.elapsed();
    let minutes = duration.as_secs() / 60;
    let seconds = duration.as_secs() % 60;
    info!(
        "Data import completed successfully: duration={:02}:{:02}",
        minutes, seconds
    );
}

pub mod location {

    use crate::eco_movement::api::response::location::LocationData;

    use super::*;
    pub struct LocationImport<'a> {
        pub eco_api: &'a EcoMovementClient,
    }

    #[async_trait]
    impl EcoImport<LocationData> for LocationImport<'_> {
        async fn fetch_page(
            &self,
            offset: usize,
        ) -> Result<ResponseData<LocationData>, reqwest::Error> {
            self.eco_api.fetch_page(Endpoint::Location, offset).await
        }

        async fn save_multiple(
            connection: &mut PgConnection,
            locations: Vec<LocationData>,
        ) -> Result<(), sqlx::Error> {
            db::location::save_multiple(connection, &locations).await
        }

        async fn truncate(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
            truncate(connection, Table::Operator).await?;
            truncate(connection, Table::Location).await?;
            Ok(())
        }
    }
}

mod connector_price {
    use crate::eco_movement::api::response::price::ConnectorPrice;

    use super::*;
    pub struct ConnectorPriceImport<'a> {
        pub eco_api: &'a EcoMovementClient,
    }

    #[async_trait]
    impl EcoImport<ConnectorPrice> for ConnectorPriceImport<'_> {
        async fn fetch_page(
            &self,
            offset: usize,
        ) -> Result<ResponseData<ConnectorPrice>, reqwest::Error> {
            self.eco_api
                .fetch_page(Endpoint::ConnectorPrice, offset)
                .await
        }

        async fn save_multiple(
            connection: &mut PgConnection,
            data: Vec<ConnectorPrice>,
        ) -> Result<(), sqlx::Error> {
            db::connector_prices::save_multiple(connection, data).await
        }

        async fn truncate(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
            truncate(connection, Table::ConnectorPrice).await
        }
    }
}

mod price {
    use crate::{
        eco_movement::{self, api::response::price::PriceData},
        ladefuchs_db,
    };

    use super::*;
    pub struct PriceImport<'a> {
        pub eco_api: &'a EcoMovementClient,
    }

    #[async_trait]
    impl EcoImport<PriceData> for PriceImport<'_> {
        async fn fetch_page(
            &self,
            offset: usize,
        ) -> Result<ResponseData<PriceData>, reqwest::Error> {
            self.eco_api.fetch_page(Endpoint::Price, offset).await
        }

        async fn save_multiple(
            connection: &mut PgConnection,
            data: Vec<PriceData>,
        ) -> Result<(), sqlx::Error> {
            db::price::save_multiple(connection, data).await
        }
        async fn truncate(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
            truncate(connection, Table::Tariff).await?;
            truncate(connection, Table::Price).await?;
            Ok(())
        }
    }

    pub async fn import(transaction: &mut PgConnection) -> Result<usize, sqlx::Error> {
        let prices = eco_movement::db::price::get_all(transaction).await?;

        ladefuchs_db::price::clear_all(transaction).await?;
        ladefuchs_db::price::save_all(transaction, &prices).await?;

        Ok(prices.len())
    }
}

async fn import<T, ImporterImpl>(
    connection: &mut PgConnection,
    importer: ImporterImpl,
    max_request_pages: u16,
) -> Result<(), eyre::ErrReport>
where
    T: DeserializeOwned + Debug,
    ImporterImpl: EcoImport<T> + Send + Sync,
{
    let mut transaction: sqlx::Transaction<'_, sqlx::Postgres> = connection.begin().await?;

    ImporterImpl::truncate(&mut transaction).await?;

    tracing::info!(
        type = any::type_name::<T>(),
        "Import data"
    );

    let stream = stream_all_data(
        |offset| importer.fetch_page(offset),
        max_request_pages.into(),
    );
    pin_mut!(stream);

    while let Some(data_result) = stream.next().await {
        let data = data_result?;
        ImporterImpl::save_multiple(&mut transaction, data).await?;
    }

    transaction.commit().await?;

    Ok(())
}

#[async_trait]
trait EcoImport<T>
where
    T: DeserializeOwned,
{
    async fn truncate(connection: &mut PgConnection) -> Result<(), sqlx::Error>;
    async fn fetch_page(&self, offset: usize) -> Result<ResponseData<T>, reqwest::Error>;
    async fn save_multiple(
        connection: &mut PgConnection,
        connector_prices: Vec<T>,
    ) -> Result<(), sqlx::Error>;
}

pub mod operator {

    use crate::{eco_movement, ladefuchs_db};
    use sqlx::PgConnection;
    pub async fn import(transaction: &mut PgConnection) -> Result<(), eyre::Error> {
        let operators = eco_movement::db::operator::get_all(transaction).await?;
        ladefuchs_db::operator::insert_or_update_operators(transaction, &operators).await?;
        Ok(())
    }
}

pub mod tariff {

    use crate::{eco_movement, ladefuchs_db};
    use sqlx::PgConnection;

    pub async fn import(transaction: &mut PgConnection) -> Result<(), sqlx::Error> {
        let tariffs = eco_movement::db::tariff::get_all(transaction).await?;
        ladefuchs_db::tariff::add_or_update_tariffs(transaction, &tariffs).await?;
        Ok(())
    }
}

pub mod dynamic_price {
    use crate::slack::{Slack, SlackClient};
    use crate::{eco_movement, ladefuchs_db};
    use sqlx::PgConnection;
    use tracing::{info, warn};

    pub async fn import(
        transaction: &mut PgConnection,
        slack: &Option<Slack>,
    ) -> Result<(), sqlx::Error> {
        info!("Importing charging locations");
        let locations = eco_movement::db::dynamic_price::get_locations(transaction).await?;

        info!(count = locations.len(), "Found charging locations");
        ladefuchs_db::dynamic_price::save_locations(transaction, &locations).await?;

        info!("Importing dynamic prices");
        let prices = eco_movement::db::dynamic_price::get_dynamic_prices(transaction).await?;

        info!(count = prices.len(), "Found dynamic prices");
        ladefuchs_db::dynamic_price::save_dynamic_prices_and_mappings(transaction, &prices).await?;

        if locations.is_empty() || prices.is_empty() {
            warn!(
                locations = locations.len(),
                prices = prices.len(),
                "Skipping stale sweep: dynamic feed empty, keeping existing rows"
            );

            slack
                .send_warning_message(
                    "Dynamic-Preisimport: leerer Feed von Eco-Movement, Sweep übersprungen und Bestand behalten."
                        .to_string(),
                )
                .await;

            return Ok(());
        }

        info!("Sweeping stale dynamic-price rows");
        ladefuchs_db::dynamic_price::sweep_stale(transaction).await?;

        Ok(())
    }
}
