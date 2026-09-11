use axum::http::StatusCode;
use ladefuchs_api::fixtures::banner::BannerBuilder;
use ladefuchs_api::fixtures::customer::CustomerBuilder;
use ladefuchs_api::fixtures::link::LinkBuilder;
use pretty_assertions::assert_eq;
use serde_json::Value;
use sqlx::PgPool;

use crate::helpers::TestClient;

#[sqlx::test]
async fn test_impression_trigger_creates_daily_row(pool: PgPool) {
    let banner = BannerBuilder::new().create(&pool).await;

    sqlx::query(
        r#"
        INSERT INTO impression_banner (banner_link, platform)
        VALUES ((SELECT id FROM link_banner WHERE pub_id = $1), 'IOS')
        "#,
    )
    .bind(banner.identifier)
    .execute(&pool)
    .await
    .unwrap();

    let row: (i32,) = sqlx::query_as(
        r#"
        SELECT count FROM impression_banner_daily
        WHERE banner_link = (SELECT id FROM link_banner WHERE pub_id = $1)
        AND platform = 'IOS' AND day = CURRENT_DATE
        "#,
    )
    .bind(banner.identifier)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(1, row.0, "expected daily count of 1");
}

#[sqlx::test]
async fn test_impression_trigger_increments_on_same_day(pool: PgPool) {
    let banner = BannerBuilder::new().create(&pool).await;

    for _ in 0..5 {
        sqlx::query(
            r#"
            INSERT INTO impression_banner (banner_link, platform)
            VALUES ((SELECT id FROM link_banner WHERE pub_id = $1), 'IOS')
            "#,
        )
        .bind(banner.identifier)
        .execute(&pool)
        .await
        .unwrap();
    }

    let row: (i64, i64) = sqlx::query_as(
        r#"
        SELECT COUNT(*) AS rows, COALESCE(SUM(count), 0) AS total
        FROM impression_banner_daily
        WHERE banner_link = (SELECT id FROM link_banner WHERE pub_id = $1)
        AND platform = 'IOS' AND day = CURRENT_DATE
        "#,
    )
    .bind(banner.identifier)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(1, row.0, "expected exactly 1 aggregated row");
    assert_eq!(5, row.1, "expected daily count of 5");
}

#[sqlx::test]
async fn test_impression_trigger_separates_platforms(pool: PgPool) {
    let banner = BannerBuilder::new().create(&pool).await;

    for _ in 0..3 {
        sqlx::query(
            r#"
            INSERT INTO impression_banner (banner_link, platform)
            VALUES ((SELECT id FROM link_banner WHERE pub_id = $1), 'IOS')
            "#,
        )
        .bind(banner.identifier)
        .execute(&pool)
        .await
        .unwrap();
    }

    for _ in 0..2 {
        sqlx::query(
            r#"
            INSERT INTO impression_banner (banner_link, platform)
            VALUES ((SELECT id FROM link_banner WHERE pub_id = $1), 'Android')
            "#,
        )
        .bind(banner.identifier)
        .execute(&pool)
        .await
        .unwrap();
    }

    let ios_count: i32 = sqlx::query_scalar(
        r#"
        SELECT count FROM impression_banner_daily
        WHERE banner_link = (SELECT id FROM link_banner WHERE pub_id = $1)
        AND platform = 'IOS' AND day = CURRENT_DATE
        "#,
    )
    .bind(banner.identifier)
    .fetch_one(&pool)
    .await
    .unwrap();

    let android_count: i32 = sqlx::query_scalar(
        r#"
        SELECT count FROM impression_banner_daily
        WHERE banner_link = (SELECT id FROM link_banner WHERE pub_id = $1)
        AND platform = 'Android' AND day = CURRENT_DATE
        "#,
    )
    .bind(banner.identifier)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(3, ios_count, "expected 3 IOS impressions");
    assert_eq!(2, android_count, "expected 2 Android impressions");
}

#[sqlx::test]
async fn test_affiliate_trigger_creates_daily_row(pool: PgPool) {
    let link = LinkBuilder::new().is_affiliate(true).create(&pool).await;
    let banner = BannerBuilder::new().link_id(link.id).create(&pool).await;

    let banner_id: i32 = sqlx::query_scalar("SELECT id FROM link_banner WHERE pub_id = $1")
        .bind(banner.identifier)
        .fetch_one(&pool)
        .await
        .unwrap();

    sqlx::query(
        r#"
        INSERT INTO affiliate_statistic (link_id, platform, link_banner_id)
        VALUES ($1, 'IOS', $2)
        "#,
    )
    .bind(link.id)
    .bind(banner_id)
    .execute(&pool)
    .await
    .unwrap();

    let row: (i32,) = sqlx::query_as(
        r#"
        SELECT count FROM affiliate_statistic_daily
        WHERE link_id = $1 AND link_banner_id = $2
        AND platform = 'IOS' AND day = CURRENT_DATE
        "#,
    )
    .bind(link.id)
    .bind(banner_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(1, row.0, "expected daily affiliate count of 1");
}

#[sqlx::test]
async fn test_affiliate_trigger_increments_on_same_key(pool: PgPool) {
    let link = LinkBuilder::new().is_affiliate(true).create(&pool).await;
    let banner = BannerBuilder::new().link_id(link.id).create(&pool).await;

    let banner_id: i32 = sqlx::query_scalar("SELECT id FROM link_banner WHERE pub_id = $1")
        .bind(banner.identifier)
        .fetch_one(&pool)
        .await
        .unwrap();

    for _ in 0..4 {
        sqlx::query(
            r#"
            INSERT INTO affiliate_statistic (link_id, platform, link_banner_id)
            VALUES ($1, 'IOS', $2)
            "#,
        )
        .bind(link.id)
        .bind(banner_id)
        .execute(&pool)
        .await
        .unwrap();
    }

    let row: (i64, i64) = sqlx::query_as(
        r#"
        SELECT COUNT(*) AS rows, COALESCE(SUM(count), 0) AS total
        FROM affiliate_statistic_daily
        WHERE link_id = $1 AND link_banner_id = $2
        AND platform = 'IOS' AND day = CURRENT_DATE
        "#,
    )
    .bind(link.id)
    .bind(banner_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(1, row.0, "expected exactly 1 aggregated row");
    assert_eq!(4, row.1, "expected daily affiliate count of 4");
}

#[sqlx::test]
async fn test_affiliate_trigger_null_banner_id(pool: PgPool) {
    let link = LinkBuilder::new().is_affiliate(true).create(&pool).await;

    for _ in 0..3 {
        sqlx::query(
            r#"
            INSERT INTO affiliate_statistic (link_id, platform, link_banner_id)
            VALUES ($1, 'IOS', NULL)
            "#,
        )
        .bind(link.id)
        .execute(&pool)
        .await
        .unwrap();
    }

    let row: (i64, i64) = sqlx::query_as(
        r#"
        SELECT COUNT(*) AS rows, COALESCE(SUM(count), 0) AS total
        FROM affiliate_statistic_daily
        WHERE link_id = $1 AND link_banner_id IS NULL
        AND platform = 'IOS' AND day = CURRENT_DATE
        "#,
    )
    .bind(link.id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        1, row.0,
        "expected 1 row for NULL banner_id (NULLS NOT DISTINCT)"
    );
    assert_eq!(3, row.1, "expected daily affiliate count of 3");
}

#[sqlx::test]
async fn test_affiliate_trigger_separates_null_and_non_null_banner(pool: PgPool) {
    let link = LinkBuilder::new().is_affiliate(true).create(&pool).await;
    let banner = BannerBuilder::new().link_id(link.id).create(&pool).await;

    let banner_id: i32 = sqlx::query_scalar("SELECT id FROM link_banner WHERE pub_id = $1")
        .bind(banner.identifier)
        .fetch_one(&pool)
        .await
        .unwrap();

    // 2 clicks with NULL banner_id
    for _ in 0..2 {
        sqlx::query(
            r#"
            INSERT INTO affiliate_statistic (link_id, platform, link_banner_id)
            VALUES ($1, 'IOS', NULL)
            "#,
        )
        .bind(link.id)
        .execute(&pool)
        .await
        .unwrap();
    }

    // 3 clicks with banner_id
    for _ in 0..3 {
        sqlx::query(
            r#"
            INSERT INTO affiliate_statistic (link_id, platform, link_banner_id)
            VALUES ($1, 'IOS', $2)
            "#,
        )
        .bind(link.id)
        .bind(banner_id)
        .execute(&pool)
        .await
        .unwrap();
    }

    let total_rows: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM affiliate_statistic_daily
        WHERE link_id = $1 AND platform = 'IOS' AND day = CURRENT_DATE
        "#,
    )
    .bind(link.id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        2, total_rows,
        "expected 2 separate rows (NULL vs non-NULL banner_id)"
    );

    let null_count: i32 = sqlx::query_scalar(
        r#"
        SELECT count FROM affiliate_statistic_daily
        WHERE link_id = $1 AND link_banner_id IS NULL
        AND platform = 'IOS' AND day = CURRENT_DATE
        "#,
    )
    .bind(link.id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let non_null_count: i32 = sqlx::query_scalar(
        r#"
        SELECT count FROM affiliate_statistic_daily
        WHERE link_id = $1 AND link_banner_id = $2
        AND platform = 'IOS' AND day = CURRENT_DATE
        "#,
    )
    .bind(link.id)
    .bind(banner_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(2, null_count, "expected 2 clicks with NULL banner_id");
    assert_eq!(3, non_null_count, "expected 3 clicks with banner_id");
}

#[sqlx::test]
async fn test_impression_api_populates_raw_and_daily_tables(pool: PgPool) {
    let banner = BannerBuilder::new().create(&pool).await;

    for _ in 0..3 {
        let request_body = serde_json::json!({
            "bannerId": banner.identifier,
            "platform": "ios"
        });
        let result = TestClient::new(pool.clone())
            .await
            .authorized()
            .post("/v3/banners/impression", request_body)
            .await;
        assert_eq!(StatusCode::OK, result.status());
    }

    // Raw table: 3 individual rows
    let raw_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM impression_banner
        WHERE banner_link = (SELECT id FROM link_banner WHERE pub_id = $1)
        "#,
    )
    .bind(banner.identifier)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        3, raw_count,
        "expected 3 individual rows in impression_banner"
    );

    // Daily table: 1 aggregated row with count=3
    let daily_count: i32 = sqlx::query_scalar(
        r#"
        SELECT count FROM impression_banner_daily
        WHERE banner_link = (SELECT id FROM link_banner WHERE pub_id = $1)
        AND platform = 'IOS' AND day = CURRENT_DATE
        "#,
    )
    .bind(banner.identifier)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(3, daily_count, "expected daily aggregation count of 3");
}

#[sqlx::test]
async fn test_impression_api_separates_platforms_in_both_tables(pool: PgPool) {
    let banner = BannerBuilder::new().create(&pool).await;

    // 2x ios, 1x android
    for platform in &["ios", "ios", "android"] {
        let request_body = serde_json::json!({
            "bannerId": banner.identifier,
            "platform": platform
        });
        let result = TestClient::new(pool.clone())
            .await
            .authorized()
            .post("/v3/banners/impression", request_body)
            .await;
        assert_eq!(StatusCode::OK, result.status());
    }

    // Raw table: 3 rows total
    let raw_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM impression_banner
        WHERE banner_link = (SELECT id FROM link_banner WHERE pub_id = $1)
        "#,
    )
    .bind(banner.identifier)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(3, raw_count, "expected 3 raw rows total");

    // Daily table: 2 rows (IOS=2, Android=1)
    let daily_rows: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM impression_banner_daily
        WHERE banner_link = (SELECT id FROM link_banner WHERE pub_id = $1)
        AND day = CURRENT_DATE
        "#,
    )
    .bind(banner.identifier)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(2, daily_rows, "expected 2 daily rows (one per platform)");
}

#[sqlx::test]
async fn test_affiliate_api_populates_raw_and_daily_tables(pool: PgPool) {
    let link = LinkBuilder::new().is_affiliate(true).create(&pool).await;
    let banner = BannerBuilder::new().link_id(link.id).create(&pool).await;

    let affiliate_url = format!(
        "/affiliate?url={}&banner={}",
        url::form_urlencoded::byte_serialize(link.source.as_bytes()).collect::<String>(),
        banner.identifier
    );

    for _ in 0..3 {
        let _result = TestClient::new(pool.clone())
            .await
            .authorized()
            .get_with_user_agent(&affiliate_url, "iPhone")
            .await;
    }

    // Raw table: 3 individual rows
    let raw_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM affiliate_statistic
        WHERE link_id = $1
        "#,
    )
    .bind(link.id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        3, raw_count,
        "expected 3 individual rows in affiliate_statistic"
    );

    // Daily table: 1 aggregated row with count=3
    let daily_count: i32 = sqlx::query_scalar(
        r#"
        SELECT count FROM affiliate_statistic_daily
        WHERE link_id = $1 AND platform = 'IOS' AND day = CURRENT_DATE
        "#,
    )
    .bind(link.id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(3, daily_count, "expected daily aggregation count of 3");
}

#[sqlx::test]
async fn test_affiliate_api_without_banner_populates_both_tables(pool: PgPool) {
    let link = LinkBuilder::new().is_affiliate(true).create(&pool).await;
    // Create a banner so the link exists, but don't include it in the affiliate URL
    let _banner = BannerBuilder::new().link_id(link.id).create(&pool).await;

    let affiliate_url = format!(
        "/affiliate?url={}",
        url::form_urlencoded::byte_serialize(link.source.as_bytes()).collect::<String>(),
    );

    for _ in 0..2 {
        let _result = TestClient::new(pool.clone())
            .await
            .authorized()
            .get_with_user_agent(&affiliate_url, "Android")
            .await;
    }

    // Raw table: 2 rows with link_banner_id = NULL
    let raw_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM affiliate_statistic
        WHERE link_id = $1 AND link_banner_id IS NULL
        "#,
    )
    .bind(link.id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(2, raw_count, "expected 2 raw rows with NULL link_banner_id");

    // Daily table: 1 row with count=2 and link_banner_id IS NULL
    let daily_count: i32 = sqlx::query_scalar(
        r#"
        SELECT count FROM affiliate_statistic_daily
        WHERE link_id = $1 AND link_banner_id IS NULL
        AND platform = 'Android' AND day = CURRENT_DATE
        "#,
    )
    .bind(link.id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        2, daily_count,
        "expected daily aggregation count of 2 for NULL banner"
    );
}

#[sqlx::test]
async fn test_customer_limit_uses_daily_aggregation(pool: PgPool) {
    let customer = CustomerBuilder::new()
        .total_impressions(5)
        .create(&pool)
        .await;

    let banner = BannerBuilder::new()
        .customer_id(customer.id)
        .create(&pool)
        .await;

    let client = TestClient::new(pool.clone()).await.authorized();

    // Post 5 impressions via API
    for _ in 0..5 {
        let request_body = serde_json::json!({
            "bannerId": banner.identifier,
            "platform": "ios"
        });
        let result = TestClient::new(pool.clone())
            .await
            .authorized()
            .post("/v3/banners/impression", request_body)
            .await;
        assert_eq!(StatusCode::OK, result.status());
    }

    // Verify the daily aggregation table has the correct count
    let daily_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(count), 0) FROM impression_banner_daily
        WHERE banner_link = (SELECT id FROM link_banner WHERE pub_id = $1)
        "#,
    )
    .bind(banner.identifier)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        5, daily_count,
        "expected 5 impressions in daily aggregation table"
    );

    // Now the banner should no longer appear in GET /v3/banners
    let result = client.get("/v3/banners").await;
    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("array");

    let ids = arr
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        !ids.contains(&banner.identifier.to_string().as_str()),
        "expected banner to be hidden after reaching impression limit, got: {json:?}"
    );
}
