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

mod connector_price;
pub mod dynamic_price;
pub mod location;
pub mod operator;
mod price;
pub mod tariff;

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
