use chrono::Utc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::state::State;

pub async fn schedule_banner_cleanup(
    scheduler: &JobScheduler,
    state: State,
) -> Result<(), eyre::Error> {
    scheduler
        .add(Job::new_async("0 0 3 * * *", move |uuid, mut lock| {
            Box::pin({
                let state_value = state.clone();

                async move {
                    info!(timestamp = %Utc::now().to_rfc3339(), "starting banner cleanup");

                    if let Err(err) = delete_marked_banners(&state_value).await {
                        error!("Could not delete marked banners: {}", err);
                    }

                    info!(timestamp = %Utc::now().to_rfc3339(), "finished banner cleanup");

                    let next_tick = lock.next_tick_for_job(uuid).await;
                    match next_tick {
                        Ok(Some(ts)) => info!("Next banner cleanup is {:?}", ts),
                        _ => error!("Could not get next tick for banner cleanup job"),
                    }
                }
            })
        })?)
        .await?;

    Ok(())
}

async fn delete_marked_banners(state: &State) -> Result<(), eyre::Error> {
    let result = sqlx::query!("DELETE FROM link_banner WHERE status = 'deleted'")
        .execute(&state.database_pool)
        .await?;

    let deleted_count = result.rows_affected();
    if deleted_count > 0 {
        info!("Deleted {} banners marked as deleted", deleted_count);
    }

    Ok(())
}
