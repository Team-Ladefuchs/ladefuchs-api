use ladefuchs_api::banner_cleanup::delete_marked_banners;
use ladefuchs_api::fixtures::banner::BannerBuilder;
use ladefuchs_api::fixtures::image::ImageBuilder;
use ladefuchs_api::fixtures::link::LinkBuilder;
use sqlx::PgPool;

async fn create_state(pool: PgPool) -> ladefuchs_api::state::State {
    let config = ladefuchs_api::config::Config {
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
    ladefuchs_api::state::State::new(pool, config)
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
