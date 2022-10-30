use std::ops::Sub;

use crate::{
    api::operator,
    db::{self, cpo, msp::save_all},
    slack::{self, MessageEmoji},
    state::State,
};
use chrono::{offset::Utc, Duration, FixedOffset};
use sqlx::Acquire;

use crate::slack::SlackClient;

pub fn spawn_price_task(duration: Duration, state: State) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        tracing::info!(
            message = format_args!(
                "Starting importer, fetching every {}h ⏰",
                duration.num_hours()
            )
        );
        let mut interval = tokio::time::interval(duration.to_std().expect("Invalid Duration"));

        loop {
            interval.tick().await;
            let offset = chrono::Duration::hours(2).num_seconds() as i32;
            let date = Utc::now() + FixedOffset::east(offset);

            let next_date = date.checked_add_signed(duration).unwrap();

            match import_prices(&state, Mode::Scheduled).await {
                Ok(_) => {
                    tracing::info!(status = "import finished 🤘");
                    tracing::info!(
                        info="fetching new data from chargeprice.app 🌐",
                        timestamp=%date.to_rfc2822()
                    );
                    tracing::info!(next_fetch =%next_date.to_rfc2822());
                }
                Err(e) => log_error("import", e.into()),
            }
        }
    })
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Scheduled,
    Manual,
}

pub async fn import_prices(state: &State, mode: Mode) -> Result<u64, eyre::Error> {
    let mut connection = state.database_pool.acquire().await?;

    let import_result = db::charge_price::import_metadata(&mut connection, 0).await?;

    if mode == Mode::Scheduled {
        if let Some(last_import) = import_result.last_import {
            if Utc::now().sub(last_import).num_hours() < 1 {
                tracing::info!(scope = "Chargeprice importer", msg = "Skipping scheduled price import because last import was last than an hour ago");
                return Ok(import_result.prices.unwrap_or_default() as u64);
            }
        }
    }

    let cpos = cpo::get_with(&mut connection, operator::Filter::Enabled).await?;
    let vehicles = db::vehicle::get_vehicles(&mut connection).await?;

    let mut current_try = 0;
    let max_tries = 3;

    tracing::info!("For {} CPOs", cpos.len());

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
                    "Chargeprice API returned zero prices :eyes: (Retries > {max_tries}), error: {error}"
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
        tokio::time::sleep(std::time::Duration::from_secs(90)).await;
    };

    let mut transaction = connection.begin().await?;
    db::charge_price::clear(&mut transaction).await?;

    let prices_count = save_all(&mut transaction, &api_results).await?;

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

    let disabled_cpos = db::cpo::hide_with_no_prices(&mut connection).await?;
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

pub fn log_error(prefix: &str, error: eyre::Error) {
    tracing::error!("{prefix}: Chargeprice API error, result={error}");
}

pub fn hours(h: u8) -> Duration {
    Duration::hours(i64::from(h))
}

pub fn spawn_cpo_task(state: State) {
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(hours(30).to_std().expect("Invalid Duration"));
        loop {
            interval.tick().await;
            if let Err(err) = import_cpos(&state).await {
                tracing::error!(task="Import CPOs", err=?err);
            };
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
