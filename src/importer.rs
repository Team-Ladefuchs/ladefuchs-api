use chrono::{offset::Utc, FixedOffset};
use sqlx::{Connection, PgConnection};
use std::{collections::HashMap, ops::Sub};

use crate::{
    charge_price_api::response::ApiResponse,
    db::{
        self,
        charge_price::{save_alle_prices, ChargePrice},
        operator,
        tariff::{save_tariffs, PriceTuple, TariffContext},
    },
    slack::{self, Emoji, SlackClient},
    state::State,
    timer::Interval,
};

pub fn spawn_price_task(state: State, mut interval: Interval) -> tokio::task::JoinHandle<()> {
    let duration = state.config.interval;
    tokio::task::spawn(async move {
        tokio::time::sleep(seconds(15)).await;
        tracing::info!(
            message = format_args!(
                "Starting importer, fetching every {}h ⏰",
                duration.num_hours()
            )
        );

        loop {
            interval.recv().await;
            {
                let offset = chrono::Duration::hours(2).num_seconds() as i32;
                let date = Utc::now() + FixedOffset::east_opt(offset).expect("invalid offset");

                tracing::info!(status = "Starting import");

                let next_date = date
                    .checked_add_signed(duration)
                    .expect("invalid date time offset");

                match import_prices_by_schedule(&state).await {
                    Ok(_) => {
                        tracing::info!(status = "Charge price import is done");
                    }
                    Err(e) => log_error("Price import", e.into()),
                }
                tracing::info!(
                    info="fetching new data from chargeprice.app 🌐",
                    timestamp=%date.to_rfc2822()
                );
                tracing::info!(next_fetch =%next_date.to_rfc2822());
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
        if self.is_import_locked() {
            tracing::warn!("Skipped import because another import is in progress");
            return Ok(0);
        }

        self.lock_import();
        let result = self
            .internal_import_prices(connection, mode, operators)
            .await;
        self.unlock_import();

        result
    }

    async fn internal_import_prices(
        &self,
        connection: &mut PgConnection,
        mode: Mode,
        operators: &[operator::admin::Operator],
    ) -> Result<usize, eyre::Error> {
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

        let mut operator_price_map: HashMap<uuid::Uuid, Vec<ChargePrice>> = HashMap::new();

        prices
            .iter()
            .filter(|price| price.blocking_fee_start > 0)
            .map(|p| p.to_owned())
            .for_each(|price| {
                operator_price_map
                    .entry(price.operator_network)
                    .or_insert(Vec::new())
                    .push(price);
            });

        let mut blocking_fee_list = self
            .charge_price_api
            .fetch_all_tariff_details(operator_price_map)
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

        if prices.is_empty() {
            transaction_prices.rollback().await?;
            let msg = "Zero prices received during last import. Current stored prices and tariffs will remain unchanged. Maybe the Chargeprice API is down.";
            tracing::warn!(msg = msg);
            let slack = &self.slack;
            slack.send(Some(Emoji::Warning), &msg).await;
            return Ok(0);
        }

        save_alle_prices(&mut transaction_prices, prices).await?;

        transaction_prices.commit().await?;

        let disabled_operators =
            db::operator::get_standard_with_no_prices(&mut *connection).await?;

        if disabled_operators.is_empty() {
            return Ok(prices_count);
        }

        let slack = &self.slack;
        slack
            .send(
                Some(Emoji::Warning),
                &format!(
                    "These standard CPOs have no prices: {} \n{}",
                    &disabled_operators.join(", "),
                    slack::MALIK
                ),
            )
            .await;

        Ok(prices_count)
    }

    async fn fetch_prices_tariffs(
        &self,
        connection: &mut PgConnection,
        operators: &[operator::admin::Operator],
        mode: Mode,
    ) -> Result<Vec<ApiResponse>, eyre::Error> {
        let vehicles = db::vehicle::get_vehicles(&mut *connection).await?;
        let mut current_try = 0;
        let max_tries = 3;
        tracing::info!(status = "Import Prices", cpos = operators.len(), %mode);
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
                    let msg = &format!(
						        "Chargeprice API returned zero prices :eyes: (Retries > {max_tries})\n{error}"
					        );
                    tracing::warn!(scope = "Chargeprice importer", msg = "Chargeprice API returned zero prices", error=%error, max_tries);
                    slack.send(Some(Emoji::Error), &msg).await;
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

async fn import_prices_by_schedule(state: &State) -> Result<usize, eyre::Error> {
    let mut connection = state.database_pool.acquire().await?;

    let import_result = db::charge_price::import_metadata(&mut connection, None).await?;

    if let Some(last_import) = import_result.last_import {
        if Utc::now().sub(last_import).num_hours() < 1 {
            tracing::info!(
                scope = "Chargeprice importer",
                msg =
                    "Skipping scheduled price import because last import was last than an hour ago"
            );
            return Ok(import_result.prices.unwrap_or_default() as usize);
        }
    }

    let operators = operator::admin::get_with(&mut *connection, operator::Filter::All).await?;
    let prices = state
        .import_prices(&mut connection, Mode::Scheduled, &operators)
        .await;

    connection.detach().close().await?;

    prices
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
        let mut interval = tokio::time::interval(hours(24));
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{config, timer};

    #[tokio::test]
    async fn test_fetch_prcies() {
        let config = config::read_config().unwrap();

        tracing::info!("Creating database pool connection");

        let (timer, _time_out) =
            timer::Timer::new(config.interval.to_std().expect("invalid interval"));

        let state = State::new(
            db::connect(&config.database_url, config.database_pool_size)
                .await
                .unwrap(),
            config.clone(),
            timer,
        );
        let mut connection = state.database_pool.acquire().await.unwrap();

        let operators = operator::get_with(&mut connection, operator::Filter::Enabled)
            .await
            .unwrap();
        let result = state
            .internal_import_prices(&mut connection, Mode::Manual, &operators)
            .await;
        if let Err(e) = &result {
            println!("{}", e.to_string());
        }
        assert!(result.is_ok());
        assert!(result.unwrap() > 0)
    }
}
