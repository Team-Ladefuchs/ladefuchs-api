use chrono::{offset::Utc, FixedOffset};
use sqlx::{Connection, PgConnection};
use std::ops::Sub;

use crate::{
    charge_price_api::client::ChargePriceAPI,
    db::{self, cpo, msp::save_all},
    slack::{self, Emoji, SlackClient},
    state::State,
    timer::Interval,
};

pub fn spawn_price_task(state: State, mut interval: Interval) -> tokio::task::JoinHandle<()> {
    let duration = state.config.interval;
    tokio::task::spawn(async move {
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
        cpos: &[cpo::CPO],
    ) -> Result<u64, eyre::Error> {
        if self.is_import_locked() {
            tracing::warn!("Skipped import because another import is in progress");
            return Ok(0);
        }

        self.lock_import();
        let result = self.internal_import_prices(connection, mode, cpos).await;
        self.unlock_import();

        result
    }

    async fn internal_import_prices(
        &self,
        connection: &mut PgConnection,
        mode: Mode,
        cpos: &[cpo::CPO],
    ) -> Result<u64, eyre::Error> {
        let vehicles = db::vehicle::get_vehicles(&mut *connection).await?;

        let mut current_try = 0;
        let max_tries = 3;

        tracing::info!(status = "Import Prices", cpos = cpos.len(), %mode);

        let api_results = loop {
            let result = self
                .charge_price_api
                .fetch_all_prices(&cpos, &vehicles)
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
                    return Ok(0);
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

        tracing::info!(status = "Writing prices to db");

        let mut transaction = connection.begin().await?;

        if cpos.len() == 1 {
            db::charge_price::clear_by_cpo(&mut transaction, cpos[0].id).await?;
        } else {
            db::charge_price::clear_all(&mut transaction).await?;
        }

        let prices_count = save_all(&mut transaction, &api_results, &self.slack).await?;

        tracing::info!(status = "Received prices", count = prices_count);
        if prices_count == 0 {
            transaction.rollback().await?;
            let msg = "Zero prices received. Current stored prices will remain unchanged";
            tracing::warn!(msg = msg);
            let slack = &self.slack;
            slack.send(Some(Emoji::Warning), &msg).await;
            return Ok(0);
        }

        tracing::info!(status = "Start fetching tariff details");

        let cpo_ids = cpos.iter().map(|c| c.id).collect::<Vec<_>>();

        match import_tariff_details(&mut transaction, &self.charge_price_api, &cpo_ids).await {
            Ok(updates) => {
                tracing::info!(
                    status = "Tariff details import done",
                    tariff_details_count = updates
                );
            }
            Err(err) => {
                log_error("Tariff details import", err.into());
            }
        }

        transaction.commit().await?;

        let disabled_cpos = db::cpo::hide_with_no_prices(&mut *connection, &cpos).await?;
        if !disabled_cpos.is_empty() {
            let slack = &self.slack;
            slack
                .send(
                    Some(Emoji::Warning),
                    &format!(
                        "These CPOs are set to be hidden, due to missing prices: {} \n{}",
                        &disabled_cpos.join(", "),
                        slack::MALIK
                    ),
                )
                .await;
        }

        Ok(prices_count)
    }
}

async fn import_tariff_details(
    transaction: &mut PgConnection,
    chargeprice_api: &ChargePriceAPI,
    cpo_ids: &[i32],
) -> Result<usize, eyre::Error> {
    let blocking_tariffs = db::tariff::get_all_blocking_fee(transaction, cpo_ids).await?;

    let blocking_fee_list = chargeprice_api
        .fetch_all_tariff_details(blocking_tariffs)
        .await?;

    for blocking_fee in &blocking_fee_list {
        blocking_fee.save(transaction).await?;
    }

    Ok(blocking_fee_list.len())
}

async fn import_prices_by_schedule(state: &State) -> Result<u64, eyre::Error> {
    let mut connection = state.database_pool.acquire().await?;

    let import_result = db::charge_price::import_metadata(&mut connection, None).await?;

    if let Some(last_import) = import_result.last_import {
        if Utc::now().sub(last_import).num_hours() < 1 {
            tracing::info!(
                scope = "Chargeprice importer",
                msg =
                    "Skipping scheduled price import because last import was last than an hour ago"
            );
            return Ok(import_result.prices.unwrap_or_default() as u64);
        }
    }

    let cpos = cpo::get_with(&mut *connection, cpo::Filter::Enabled).await?;

    let prices = state
        .import_prices(&mut connection, Mode::Scheduled, &cpos)
        .await;

    connection.detach().close().await?;

    prices
}

pub fn log_error(prefix: &str, error: eyre::Error) {
    tracing::error!("{prefix}: Chargeprice API error, result={error}");
}

pub const fn hours(h: u8) -> std::time::Duration {
    std::time::Duration::from_secs(3600 * h as u64)
}

pub fn spawn_cpo_task(state: State) {
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(hours(30));
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
    let mut transition = connection.begin().await?;
    let companies = state.charge_price_api.fetch_operator().await?;
    db::cpo_cache::clear(&mut transition).await?;
    db::cpo_cache::save_all_operator(&mut transition, &companies).await?;

    let charge_stations = state
        .charge_price_api
        .fetch_operator_charging_stations()
        .await?;
    db::cpo_cache::update_charge_stations_statistics(&mut transition, charge_stations).await?;

    transition.commit().await?;
    connection.detach().close().await?;

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

        let cpos = cpo::get_with(&mut connection, cpo::Filter::Enabled)
            .await
            .unwrap();
        let result = state
            .internal_import_prices(&mut connection, Mode::Manual, &cpos)
            .await;
        if let Err(e) = &result {
            println!("{}", e.to_string());
        }
        assert!(result.is_ok());
        assert!(result.unwrap() > 0)
    }
}
