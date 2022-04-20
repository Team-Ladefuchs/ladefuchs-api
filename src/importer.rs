use crate::{
    charge_price_api::{self, response::AllChargePrices},
    db::msp::save_all,
    state::State,
};
use chrono::Duration;
use sqlx::Pool;
use tokio::time;

pub fn spawn_background_task(duration: Duration, state: State) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        time::sleep(Duration::seconds(3).to_std().unwrap()).await;
        tracing::info!(
            message = format_args!(
                "Starting importer, fetching every {}h ⏰",
                duration.num_hours()
            )
        );
        let mut interval = tokio::time::interval(duration.to_std().expect("Invalid Duration"));
        loop {
            interval.tick().await;
            let date = chrono::offset::Utc::now();

            let next_date = date.checked_add_signed(duration).unwrap();
            match charge_price_api::client::fetch_data(&state).await {
                Ok(results) => {
                    match import(results, &state.database_pool).await {
                        Ok(_) => {
                            tracing::info!(status = "🤘 work done 🤘");
                            tracing::info!(
                                info="fetching new data from chargeprice.app 🌐",
                                timestamp=%date.to_rfc2822()
                            );
                            tracing::info!(next_fetch =%next_date.to_rfc2822());
                        }
                        Err(e) => log_error(e.into()),
                    };
                }
                Err(err) => log_error(err),
            };
        }
    })
}

pub async fn import(
    results: AllChargePrices,
    pool: &Pool<sqlx::Postgres>,
) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    for api_result in results {
        save_all(
            &mut transaction,
            &api_result.msps,
            api_result.vehicle_id,
            api_result.cpo_id,
        )
        .await?
    }
    transaction.commit().await?;
    Ok(())
}

fn log_error(err: eyre::Error) {
    tracing::error!("Import error: {}", err);
}

pub fn hours(h: u8) -> Duration {
    Duration::hours(i64::from(h))
}
