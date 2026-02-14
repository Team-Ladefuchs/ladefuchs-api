use std::path::PathBuf;

use chrono::Utc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

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

#[derive(Debug)]
struct DeletedBanner {
    id: i32,
    link_id: i32,
    image_id: Option<i32>,
    file_path: Option<String>,
}

pub async fn delete_marked_banners(state: &State) -> Result<(), eyre::Error> {
    let banners: Vec<DeletedBanner> = sqlx::query_as!(
        DeletedBanner,
        r#"
        SELECT
            link_banner.id,
            link_banner.link_id,
            link_banner.image as image_id,
            i.file_path
        FROM link_banner
        LEFT JOIN image i ON link_banner.image = i.id
        WHERE link_banner.status = 'deleted'
        "#
    )
    .fetch_all(&state.database_pool)
    .await?;

    if banners.is_empty() {
        return Ok(());
    }

    let mut deleted_count = 0;

    for banner in &banners {
        // Check if the image is used by other banners
        let should_delete_image = if let Some(image_id) = banner.image_id {
            let count: i64 = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM link_banner WHERE image = $1 AND id != $2",
                image_id,
                banner.id
            )
            .fetch_one(&state.database_pool)
            .await?
            .unwrap_or(0);

            if count > 0 {
                // Image is used by other banners, just set it to NULL
                sqlx::query!(
                    "UPDATE link_banner SET image = NULL WHERE id = $1",
                    banner.id
                )
                .execute(&state.database_pool)
                .await?;

                false
            } else {
                true
            }
        } else {
            false
        };

        // Check if the link is used by other banners
        let should_delete_link = {
            let count: i64 = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM link_banner WHERE link_id = $1 AND id != $2",
                banner.link_id,
                banner.id
            )
            .fetch_one(&state.database_pool)
            .await?
            .unwrap_or(0);

            count == 0
        };

        // Delete the banner
        sqlx::query!("DELETE FROM link_banner WHERE id = $1", banner.id)
            .execute(&state.database_pool)
            .await?;

        // Delete the image if it's not used by other banners
        if should_delete_image {
            if let Some(image_id) = banner.image_id {
                sqlx::query!("DELETE FROM image WHERE id = $1", image_id)
                    .execute(&state.database_pool)
                    .await?;
            }

            if let Some(file_path) = banner.file_path.as_deref() {
                let path = PathBuf::from(file_path);
                if let Err(err) = tokio::fs::remove_file(&path).await {
                    warn!("Could not delete banner file {}: {}", file_path, err);
                }
            }
        }

        // Delete the link if it's not used by other banners
        if should_delete_link {
            sqlx::query!("DELETE FROM link WHERE id = $1", banner.link_id)
                .execute(&state.database_pool)
                .await?;
        }

        deleted_count += 1;
    }

    info!("Deleted {} banners", deleted_count);

    Ok(())
}
