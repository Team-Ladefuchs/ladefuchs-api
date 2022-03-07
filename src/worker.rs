use chrono::{Duration, Local};

use crate::{charge_price_api, db, state::State};

pub fn spaw_import_task(duration: Duration, state: State) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(duration.to_std().unwrap());
        loop {
            interval.tick().await;
            let date = Local::now();

            let next_date = date.checked_add_signed(duration).unwrap();
            tracing::info!(next_fetch=%next_date.to_rfc3339());
            match charge_price_api::fetch_data(&state).await {
                Ok(results) => {
                    match db::import(results, &state.database_pool).await {
                        Ok(_) => {
                            tracing::info!("import success!");
                            // duration.get
                            tracing::info!(
                                info="fetching new data from chargeprice.app",
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
