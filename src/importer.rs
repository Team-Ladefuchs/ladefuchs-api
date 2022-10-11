use std::sync::Arc;

use crate::{
    api::operator,
    charge_price_api::client::ChargePriceAPI,
    db::{self, cpo, msp::save_all},
    slack::{self, MessageEmoji},
    state::State,
};
use chrono::{Duration, FixedOffset};
use sqlx::Acquire;

use crate::slack::SlackClient;

pub fn spawn_background_task(duration: Duration, state: State) -> tokio::task::JoinHandle<()> {
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
            let date = chrono::offset::Utc::now() + FixedOffset::east(offset);

            let next_date = date.checked_add_signed(duration).unwrap();

            match import(&state, Mode::Scheduled).await {
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

pub async fn import(state: &State, mode: Mode) -> Result<u32, eyre::Error> {
    let client = Arc::new(ChargePriceAPI::new(&state.config)?);
    let mut connection = state.database_pool.acquire().await?;

    let cpos = cpo::get_with(&mut connection, operator::Filter::Enabled).await?;
    let vehicles = db::vehicle::get_vehicles(&mut connection).await?;

    let mut current_try = 0;
    let max_tries = 3;

    tracing::info!("For {} CPOs", cpos.len());

    let api_results = loop {
        let result = ChargePriceAPI::fetch_prices(&client, &cpos, &vehicles).await;

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
