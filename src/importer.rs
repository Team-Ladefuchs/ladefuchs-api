use chrono::offset::Utc;
use sqlx::{Connection, PgConnection};
use std::ops::Sub;

use crate::{
    charge_price_api::response::condition::ApiPriceResponse,
    db::{
        self,
        charge_price::{save_alle_prices, ChargePrice},
        operator,
        tariff::{save_tariffs, PriceTuple, TariffContext},
    },
    slack::SlackClient,
    state::State,
    timer::Interval,
};

pub fn spawn_price_task(state: State, mut interval: Interval) -> tokio::task::JoinHandle<()> {
    let duration = state.config.interval;
    tokio::task::spawn(async move {
        tokio::time::sleep(seconds(15)).await;
        tracing::info!(
            status = "Import task started",
            interval = format!("{}h ⏰", duration.num_hours())
        );

        loop {
            interval.recv().await;
            {
                tracing::info!(status = "Starting price import");
                match import_prices_by_schedule(&state).await {
                    Ok(_) => {
                        tracing::info!(status = "Charge price import is done");
                    }
                    Err(e) => log_error("Price import", e.into()),
                }
                tracing::info!(
                    status = format!(
                        "Next import from from chargeprice.app 🌐 in {}h ⏰",
                        duration.num_hours()
                    )
                );
            }
        }
    })
}

#[derive(Clone, Debug, PartialEq, strum_macros::Display)]
pub enum Mode {
    Scheduled,
    Manual,
}

impl State {
    pub async fn import_prices(
        &self,
        connection: &mut PgConnection,
        mode: Mode,
        operators: &[operator::admin::Operator],
    ) -> Result<usize, eyre::Error> {
        let Some(_lock) = self.lock() else {
            tracing::info!("Skipped import because another import is in progress");
            return Ok(0);
        };

        let api_results = self
            .fetch_prices_tariffs(connection, operators, mode)
            .await?;

        let mut transaction = connection.begin().await?;

        let mut prices = save_tariffs(TariffContext {
            transaction: &mut transaction,
            slack: &self.slack,
            responses: &api_results,
        })
        .await?;

        transaction.commit().await?;

        let prices_count = prices.len();
        tracing::info!(status = "Received prices", count = prices_count);

        let mut blocking_fee_list = self
            .charge_price_api
            .fetch_all_tariff_details(&prices)
            .await?;

        for chargeprice in prices.iter_mut() {
            if let Some(blocking_fee) = blocking_fee_list.remove(&PriceTuple(
                chargeprice.operator_network,
                chargeprice.tariff_relation,
                chargeprice.c_type,
            )) {
                chargeprice.blocking_fee = blocking_fee;
            }
        }

        tracing::info!(status = "Writing charge prices to database");
        let mut transaction_prices = connection.begin().await?;

        if operators.len() == 1 {
            db::charge_price::clear_by_operator(&mut transaction_prices, operators[0].id).await?;
        } else {
            db::charge_price::clear_all(&mut transaction_prices).await?;
        }

        if prices.is_empty() || self.only_one_tariff(&prices) {
            transaction_prices.rollback().await?;
            let msg = "Zero prices from Chargeprice received during last import. Current stored prices and tariffs will remain unchanged. Maybe the Chargeprice API is down.";
            tracing::warn!(msg = msg);
            let slack = &self.slack;
            slack.send_warning_message(msg.to_string()).await;
            return Ok(0);
        }

        save_alle_prices(&mut transaction_prices, prices).await?;

        transaction_prices.commit().await?;

        let disabled_operators =
            db::operator::get_standard_with_no_prices(&mut *connection).await?;

        if disabled_operators.is_empty() {
            return Ok(prices_count);
        }

        let message = format!(
            "These standard CPOs have no prices:\n{}",
            &disabled_operators.join(", "),
        );
        tracing::warn!(message);
        if let Some(slack) = &self.slack {
            slack.send_warning_message(message).await;
        }

        Ok(prices_count)
    }

    fn only_one_tariff(&self, prices: &[ChargePrice]) -> bool {
        if let Some(first_id) = prices.first().map(|cp| cp.tariff_id) {
            prices.iter().all(|p| p.tariff_id == first_id)
        } else {
            true
        }
    }

    async fn fetch_prices_tariffs(
        &self,
        connection: &mut PgConnection,
        operators: &[operator::admin::Operator],
        mode: Mode,
    ) -> Result<Vec<ApiPriceResponse>, eyre::Error> {
        let vehicles = db::vehicle::get_vehicles(&mut *connection).await?;
        let mut current_try = 0;
        let max_tries = 3;
        tracing::info!(status = "Import psrices", cpos = operators.len(), %mode);
        let api_results = loop {
            let result = self
                .charge_price_api
                .fetch_all_prices(&operators, &vehicles)
                .await;

            current_try += 1;

            match result {
                Ok(value) => break value,
                Err(error) if mode == Mode::Manual => return Err(error),
                Err(error) if current_try > max_tries => {
                    let slack = &self.slack;
                    let message = format!(
						        "Something went wrong while fetching prices Chargeprice API :eyes: (Retries > {max_tries})\n{error}"
					        );
                    tracing::warn!(scope = "Chargeprice importer", msg = "Something went wrong while fetching prices Chargeprice API", error=%error, max_tries);
                    slack.send_error_message(message).await;
                    return Ok(vec![]);
                }
                Err(error) => {
                    tracing::error!(msg = "Got an error while fetching prices", ?error,);
                    tracing::error!(?error);
                    tracing::warn!(
                        "Retry({current_try}) fetching prices from Chargeprice after 90s break."
                    )
                }
            };
            tracing::info!(status = "sleeping for 90s");
            tokio::time::sleep(std::time::Duration::from_secs(90)).await;
        };
        Ok(api_results)
    }
}

async fn import_prices_by_schedule(state: &State) -> Result<(), eyre::Error> {
    let mut connection = state.database_pool.acquire().await?;

    let import_result = db::charge_price::last_import_context(&mut connection, None).await?;

    if let Some(last_import) = import_result.last_import {
        if Utc::now().sub(last_import).num_hours() < 1 {
            tracing::info!(
                status =
                    "Skipping scheduled price import because last import was last than an hour ago"
            );
            return Ok(());
        }
    }

    let operators = operator::admin::get_with(&mut *connection, operator::Filter::Enabled).await?;
    state
        .import_prices(&mut connection, Mode::Scheduled, &operators)
        .await?;

    let tariff_count = db::tariff::get_count(&mut *connection).await?;
    tracing::info!(status = "Check tariffs", "count" = tariff_count);

    Ok(())
}

pub fn log_error(prefix: &str, error: eyre::Error) {
    tracing::error!("{prefix}: Chargeprice API error, result={error}");
}

pub const fn hours(h: u64) -> std::time::Duration {
    std::time::Duration::from_secs(3600 * h)
}

pub const fn seconds(s: u64) -> std::time::Duration {
    std::time::Duration::from_secs(s)
}

pub fn spawn_operator_task(state: State) {
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(hours(23));
        loop {
            interval.tick().await;
            {
                if let Err(err) = import_operators(&state).await {
                    tracing::error!(task="Error import operator and charging stations statistic", err=?err);
                };
                tracing::info!(status = "Import operator and charging stations job complete");
            }
        }
    });
}

async fn import_operators(state: &State) -> Result<(), eyre::Report> {
    let mut connection = state.as_ref().database_pool.acquire().await?;
    let mut transaction = connection.begin().await?;
    let companies = state.charge_price_api.fetch_operator().await?;

    db::operator::insert_or_update_companies(&mut transaction, &companies).await?;
    transaction.commit().await?;

    let mut transaction_stations = connection.begin().await?;
    let charge_stations = state
        .charge_price_api
        .fetch_operator_charging_stations()
        .await?;
    db::operator::update_charge_stations_statistics(&mut transaction_stations, charge_stations)
        .await?;

    transaction_stations.commit().await?;

    Ok(())
}
