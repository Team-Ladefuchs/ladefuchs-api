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

            match import(&state).await {
                Ok(_) => {
                    tracing::info!(status = "🤘 work done 🤘");
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

pub async fn import(state: &State) -> Result<(), eyre::Error> {
    let client = Arc::new(ChargePriceAPI::new(&state.config)?);
    let mut connection = state.database_pool.acquire().await?;

    let cpos = cpo::get_with(&mut connection, operator::Filter::All).await?;
    let vehicles = db::vehicle::get_vehicles(&mut connection).await?;

    let mut tries = 3;

    let result = loop {
        let result = ChargePriceAPI::fetch_data(&client, &cpos, &vehicles).await?;
        if result.charge_point_prices > 0 {
            break result;
        }
        tries -= 1;

        if tries == 0 {
            let slack = &state.slack;
            let msg = &format!("Chargeprice API returned zero prices :eyes: (Max retries > 3");
            tracing::warn!(msg = msg);
            slack.send(None, &msg).await;
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    };

    let mut transaction = connection.begin().await?;
    db::charge_price::clear(&mut transaction).await?;

    tracing::info!("Received prices: {} ", result.charge_point_prices);
    for api_result in result.responses {
        save_all(&mut transaction, &api_result.msps, api_result.cpo_id).await?
    }

    transaction.commit().await?;

    let disabled_cpos = db::cpo::disable_with_no_prices(&mut connection).await?;
    if !disabled_cpos.is_empty() {
        let slack = &state.slack;
        slack
            .send(
                Some(MessageEmoji::Warning),
                &format!(
                    "These CPOs were deactivated due to missing prices: {} \n{}",
                    &disabled_cpos.join(", "),
                    slack::MALIK
                ),
            )
            .await;
    }
    Ok(())
}

pub fn log_error(prefix: &str, error: eyre::Error) {
    tracing::error!("{prefix}: Chargeprice API error, result={error}");
}

pub fn hours(h: u8) -> Duration {
    Duration::hours(i64::from(h))
}
