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

pub(crate) async fn delete_marked_banners(state: &State) -> Result<(), eyre::Error> {
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

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use crate::fixtures::banner::BannerBuilder;
    use crate::fixtures::image::ImageBuilder;
    use crate::fixtures::link::LinkBuilder;
    use sqlx::PgPool;

    async fn create_state(pool: PgPool) -> crate::state::State {
        let config = crate::config::Config {
            database_url: "postgres://localhost/ladefuchs_test".parse().unwrap(),
            database_pool_size: 5,
            eco_movement_api_key: "".to_owned(),
            eco_movement_api_url: "https://example.com/".parse().unwrap(),
            port: 3000,
            listen: [127, 0, 0, 1].into(),
            cron_schedule: "0 45 23 * * *".to_owned(),
            domain: "http://127.0.0.1:3000".parse().unwrap(),
            slack_channel: None,
            slack_token: None,
            admin_user: None,
            admin_pwd: None,
            admin_domain: "http://127.0.0.1:8080".parse().unwrap(),
            docs_dir: std::path::PathBuf::from("./docs"),
            import_on_start: false,
            max_request_pages: 1000,
        };
        crate::state::State::new(pool, config)
    }

    #[sqlx::test]
    async fn test_delete_marked_banners_deletes_banner_with_deleted_status(pool: PgPool) {
        let banner = BannerBuilder::new().status("deleted").create(&pool).await;

        let state = create_state(pool.clone()).await;
        delete_marked_banners(&state).await.unwrap();

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM link_banner WHERE pub_id = $1")
                .bind(banner.identifier)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(0, count, "expected banner to be deleted");
    }

    #[sqlx::test]
    async fn test_delete_marked_banners_deletes_image_when_not_shared(pool: PgPool) {
        let image = ImageBuilder::new().create(&pool).await;
        let _banner = BannerBuilder::new()
            .image(image.id)
            .status("deleted")
            .create(&pool)
            .await;

        let state = create_state(pool.clone()).await;
        delete_marked_banners(&state).await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM image WHERE id = $1")
            .bind(image.id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(0, count, "expected image to be deleted when not shared");
    }

    #[sqlx::test]
    async fn test_delete_marked_banners_keeps_image_when_shared(pool: PgPool) {
        let image = ImageBuilder::new().create(&pool).await;

        // Create two banners sharing the same image, mark only the first as deleted
        let _banner1 = BannerBuilder::new()
            .image(image.id)
            .status("deleted")
            .create(&pool)
            .await;
        let _banner2 = BannerBuilder::new().image(image.id).create(&pool).await;

        let state = create_state(pool.clone()).await;
        delete_marked_banners(&state).await.unwrap();

        // Verify deleted banners are gone
        let deleted_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM link_banner WHERE status = 'deleted'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(0, deleted_count, "expected deleted banner to be removed");

        // Verify the image is NOT deleted (it's shared)
        let image_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM image WHERE id = $1")
            .bind(image.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(1, image_count, "expected image to be kept when shared");
    }

    #[sqlx::test]
    async fn test_delete_marked_banners_deletes_link_when_not_shared(pool: PgPool) {
        let link = LinkBuilder::new().create(&pool).await;
        let _banner = BannerBuilder::new()
            .link_id(link.id)
            .status("deleted")
            .create(&pool)
            .await;

        let state = create_state(pool.clone()).await;
        delete_marked_banners(&state).await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM link WHERE id = $1")
            .bind(link.id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(0, count, "expected link to be deleted when not shared");
    }

    #[sqlx::test]
    async fn test_delete_marked_banners_keeps_link_when_shared(pool: PgPool) {
        let link = LinkBuilder::new().create(&pool).await;

        // Create two banners sharing the same link, mark only the first as deleted
        let _banner1 = BannerBuilder::new()
            .link_id(link.id)
            .status("deleted")
            .create(&pool)
            .await;
        let _banner2 = BannerBuilder::new().link_id(link.id).create(&pool).await;

        let state = create_state(pool.clone()).await;
        delete_marked_banners(&state).await.unwrap();

        // Verify deleted banners are gone
        let deleted_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM link_banner WHERE status = 'deleted'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(0, deleted_count, "expected deleted banner to be removed");

        // Verify the link is NOT deleted (it's shared)
        let link_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM link WHERE id = $1")
            .bind(link.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(1, link_count, "expected link to be kept when shared");
    }

    #[sqlx::test]
    async fn test_delete_marked_banners_does_nothing_when_no_deleted_banners(pool: PgPool) {
        let banner = BannerBuilder::new().create(&pool).await;

        let state = create_state(pool.clone()).await;
        delete_marked_banners(&state).await.unwrap();

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM link_banner WHERE pub_id = $1")
                .bind(banner.identifier)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(1, count, "expected banner to still exist");
    }

    #[sqlx::test]
    async fn test_delete_marked_banners_handles_multiple_deleted_banners(pool: PgPool) {
        let banner1 = BannerBuilder::new().status("deleted").create(&pool).await;
        let banner2 = BannerBuilder::new().status("deleted").create(&pool).await;
        let banner3 = BannerBuilder::new().create(&pool).await;

        let state = create_state(pool.clone()).await;
        delete_marked_banners(&state).await.unwrap();

        // Verify banner1 and banner2 are deleted
        let count1: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM link_banner WHERE pub_id = $1")
                .bind(banner1.identifier)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(0, count1, "expected banner1 to be deleted");

        let count2: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM link_banner WHERE pub_id = $1")
                .bind(banner2.identifier)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(0, count2, "expected banner2 to be deleted");

        // Verify banner3 still exists
        let count3: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM link_banner WHERE pub_id = $1")
                .bind(banner3.identifier)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(1, count3, "expected banner3 to still exist");
    }
}
