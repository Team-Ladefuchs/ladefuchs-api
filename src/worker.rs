use chrono::Duration;

use crate::{charge_price_api, db, state::State};

pub fn spaw_import_task(duration: Duration, state: State) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        tracing::info!(mesage = format_args!("fetching every {}h ⏰", duration.num_hours()));
        let mut interval = tokio::time::interval(duration.to_std().expect("Invalid Duration"));
        loop {
            interval.tick().await;
            let date = chrono::offset::Utc::now();

            let next_date = date.checked_add_signed(duration).unwrap();
            match charge_price_api::fetch_data(&state).await {
                Ok(results) => {
                    match db::import(results, &state.database_pool).await {
                        Ok(_) => {
                            tracing::info!(status = "🤘 work done 🤘");
                            // duration.get
                            tracing::info!(
                                info="fetching new data from chargeprice.app 🌐",
                                timestamp=%date.to_rfc3339()
                            );
                            tracing::info!(next_fetch=%next_date.to_rfc3339());
                        }
                        Err(e) => log_error(e.into()),
                    };
                }
                Err(err) => log_error(err),
            };
        }
    })
}

fn log_error(err: eyre::Error) {
    tracing::error!("Import error: {}", err);
}

pub fn hours(h: u8) -> Duration {
    Duration::hours(i64::from(h))
}
