use std::ops::Sub;

use crate::{
    db::{self, cpo, msp::save_all},
    slack::{self, MessageEmoji},
    state::State,
    timer::Interval,
};
use chrono::{offset::Utc, FixedOffset};
use sqlx::{pool::PoolConnection, Acquire, Postgres};

use crate::slack::SlackClient;

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

            match import_tariff_details(&state).await {
                Ok(updates) => {
                    tracing::info!(
                        status = "Tariff details import done",
                        tariffs_count = updates
                    );
                }
                Err(err) => {
                    log_error("Tariff details import", err.into());
                }
            }

            tracing::info!(
                info="fetching new data from chargeprice.app 🌐",
                timestamp=%date.to_rfc2822()
            );
            tracing::info!(next_fetch =%next_date.to_rfc2822());
        }
    })
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Scheduled,
    Manual,
}

pub async fn import_prices(
    state: &State,
    connection: &mut PoolConnection<Postgres>,
    mode: Mode,
    cpos: &[cpo::CPO],
) -> Result<u64, eyre::Error> {
    let vehicles = db::vehicle::get_vehicles(&mut *connection).await?;

    let mut current_try = 0;
    let max_tries = 3;
    tracing::info!("Import Prices for {} CPOs", cpos.len());

    let api_results = loop {
        let result = state
            .charge_price_api
            .fetch_all_prices(&cpos, &vehicles)
            .await;

        current_try += 1;

        match result {
            Ok(value) => break value,
            Err(error) if mode == Mode::Manual => return Err(error),
            Err(error) if current_try > max_tries => {
                let slack = &state.slack;
                let msg = &format!(
                    "Chargeprice API returned zero prices :eyes: (Retries > {max_tries})\n, error: {error}"
                );
                tracing::warn!(scope = "Chargeprice importer", msg = msg);
                slack.send(Some(MessageEmoji::Error), &msg).await;
                return Ok(0);
            }
            _ => {
                tracing::warn!(
                    "Retry({current_try}) fetching prices from Chargeprice after 90s break."
                )
            }
        };
        tracing::info!(status = "sleeping for 90s");
        tokio::time::sleep(std::time::Duration::from_secs(90)).await;
    };

    let mut transaction = connection.begin().await?;
    if cpos.len() == 1 {
        db::charge_price::clear_by_cpo(&mut transaction, cpos[0].id).await?;
    } else {
        db::charge_price::clear_all(&mut transaction).await?;
    }

    let prices_count = save_all(&mut transaction, &api_results, &state.slack).await?;

    tracing::info!("Received prices: {prices_count}");
    if prices_count == 0 {
        transaction.rollback().await?;
        let msg = "Zero prices received. Current stored prices will remain unchanged";
        tracing::warn!(msg = msg);
        let slack = &state.slack;
        slack.send(Some(MessageEmoji::Warning), &msg).await;
        return Ok(0);
    }
    transaction.commit().await?;

    let disabled_cpos = db::cpo::hide_with_no_prices(&mut *connection, &cpos).await?;
    if !disabled_cpos.is_empty() {
        let slack = &state.slack;
        slack
            .send(
                Some(MessageEmoji::Warning),
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

async fn import_tariff_details(state: &State) -> Result<usize, eyre::Error> {
    let mut connection = state.database_pool.acquire().await?;
    let blocking_tariffs = db::tariff::get_all_blocking_fee(&mut connection).await?;

    let blocking_fee_list = state
        .as_ref()
        .charge_price_api
        .fetch_all_tariff_details(blocking_tariffs)
        .await?;

    let mut transaction = connection.begin().await?;
    for blocking_fee in &blocking_fee_list {
        blocking_fee.save(&mut transaction).await?;
    }
    transaction.commit().await?;

    Ok(blocking_fee_list.len())
}

async fn import_prices_by_schedule(state: &State) -> Result<u64, eyre::Error> {
    let mut connection = state.database_pool.acquire().await?;

    let import_result =
        db::charge_price::import_metadata(&mut connection, chrono::Duration::hours(0)).await?;

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

    let cpos = cpo::get_with(&mut connection, cpo::Filter::Enabled).await?;
    import_prices(&state, &mut connection, Mode::Scheduled, &cpos).await
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
            if let Err(err) = import_cpos(&state).await {
                tracing::error!(task="Import CPOs", err=?err);
            };
            tracing::info!(status = "CPO import job complete");
        }
    });
}

async fn import_cpos(state: &State) -> Result<(), eyre::Report> {
    let mut connection = state.as_ref().database_pool.acquire().await?;
    let mut trx = connection.begin().await?;
    let companies = state.charge_price_api.fetch_companies().await?;

    db::cpo_cache::clear(&mut trx).await?;
    db::cpo_cache::save_all(&mut trx, &companies).await?;

    trx.commit().await?;

    Ok(())
}
