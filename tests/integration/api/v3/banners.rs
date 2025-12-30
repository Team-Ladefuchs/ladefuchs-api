use axum::http::StatusCode;
use chrono::Utc;
use ladefuchs_api::fixtures::banner::BannerBuilder;
use pretty_assertions::assert_eq;
use serde_json::Value;
use sqlx::PgPool;

use crate::helpers::TestClient;

#[sqlx::test]
async fn test_banners_v3_returns_200_and_json_array(pool: PgPool) {
    let banner = BannerBuilder::new().create(&pool).await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/banners")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_array(),
        "expected response body to be a JSON array, got: {json:?}"
    );

    let arr = json.as_array().expect("already asserted is_array");
    assert!(
        !arr.is_empty(),
        "expected non-empty array response, got: {json:?}"
    );

    let ids = arr
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        ids.contains(&banner.identifier.to_string().as_str()),
        "expected banner to be included in response, got: {json:?}"
    );

    let first = arr.first().expect("non-empty array");
    assert!(
        first.get("identifier").is_some(),
        "expected `identifier` field in item, got: {first:?}"
    );
    assert!(
        first.get("affiliateLinkUrl").is_some(),
        "expected `affiliateLinkUrl` field in item, got: {first:?}"
    );
    assert!(
        first.get("imageUrl").is_some(),
        "expected `imageUrl` field in item, got: {first:?}"
    );
    assert!(
        first.get("frequency").is_some(),
        "expected `frequency` field in item, got: {first:?}"
    );
    assert!(
        first.get("isAffiliate").is_some(),
        "expected `isAffiliate` field in item, got: {first:?}"
    );
    assert!(
        first.get("lastUpdatedDate").is_some(),
        "expected `lastUpdatedDate` field in item, got: {first:?}"
    );
    let last_updated = first
        .get("lastUpdatedDate")
        .and_then(|v| v.as_str())
        .expect("lastUpdatedDate should be a string");
    assert!(
        !last_updated.is_empty(),
        "expected lastUpdatedDate to be a non-empty ISO 8601 string, got: {last_updated:?}"
    );
}

#[sqlx::test]
async fn test_banners_v3_returns_empty_array_when_no_banners_exist(pool: PgPool) {
    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/banners")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(json.is_array(), "expected array, got: {json:?}");
    assert_eq!(
        0,
        json.as_array().expect("array").len(),
        "expected empty list when no banners exist, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_banners_v3_filters_out_expired_banners(pool: PgPool) {
    let now = Utc::now();
    let expired_banner = BannerBuilder::new()
        .expiration(now - chrono::Duration::days(1))
        .starts(now - chrono::Duration::days(10))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/banners")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(json.is_array(), "expected array, got: {json:?}");
    let arr = json.as_array().expect("array");

    let ids = arr
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        !ids.contains(&expired_banner.identifier.to_string().as_str()),
        "expected expired banner to not be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_banners_v3_filters_out_future_banners(pool: PgPool) {
    let now = Utc::now();
    let future_banner = BannerBuilder::new()
        .starts(now + chrono::Duration::days(1))
        .expiration(now + chrono::Duration::days(10))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/banners")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(json.is_array(), "expected array, got: {json:?}");
    let arr = json.as_array().expect("array");

    let ids = arr
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        !ids.contains(&future_banner.identifier.to_string().as_str()),
        "expected future banner to not be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_banners_v3_includes_active_banners(pool: PgPool) {
    let now = Utc::now();
    let active_banner = BannerBuilder::new()
        .starts(now - chrono::Duration::days(1))
        .expiration(now + chrono::Duration::days(10))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/banners")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(json.is_array(), "expected array, got: {json:?}");
    let arr = json.as_array().expect("array");

    let ids = arr
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        ids.contains(&active_banner.identifier.to_string().as_str()),
        "expected active banner to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_banners_v3_uses_v3_image_path_format(pool: PgPool) {
    let _banner = BannerBuilder::new().create(&pool).await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/banners")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(json.is_array(), "expected array, got: {json:?}");
    let arr = json.as_array().expect("array");

    assert!(
        !arr.is_empty(),
        "expected at least one banner to verify image path format, got: {json:?}"
    );

    for banner_item in arr {
        let image_url = banner_item
            .get("imageUrl")
            .and_then(|v| v.as_str())
            .expect("imageUrl should be a string");

        assert!(
            image_url.contains("/image/"),
            "expected v3 image URL to use /image/ path, got: {image_url:?}"
        );
        assert!(
            !image_url.contains("/img/banner/"),
            "expected v3 image URL to not use /img/banner/ path, got: {image_url:?}"
        );
    }
}

#[sqlx::test]
async fn test_banners_v3_impression_post_returns_200_for_ios_platform(pool: PgPool) {
    let banner = BannerBuilder::new().create(&pool).await;

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

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM impression_banner WHERE banner_link = (SELECT id FROM link_banner WHERE pub_id = $1) AND platform = 'IOS'"
    )
    .bind(banner.identifier)
    .fetch_one(&pool)
    .await
    .expect("could not query impression");

    assert_eq!(
        1, count,
        "expected one impression to be inserted, got: {count}"
    );
}

#[sqlx::test]
async fn test_banners_v3_impression_post_returns_200_for_android_platform(pool: PgPool) {
    let banner = BannerBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "bannerId": banner.identifier,
        "platform": "android"
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/banners/impression", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM impression_banner WHERE banner_link = (SELECT id FROM link_banner WHERE pub_id = $1) AND platform = 'Android'"
    )
    .bind(banner.identifier)
    .fetch_one(&pool)
    .await
    .expect("could not query impression");

    assert_eq!(
        1, count,
        "expected one impression to be inserted, got: {count}"
    );
}

#[sqlx::test]
async fn test_banners_v3_impression_post_returns_200_for_web_platform(pool: PgPool) {
    let banner = BannerBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "bannerId": banner.identifier,
        "platform": "web"
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/banners/impression", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM impression_banner WHERE banner_link = (SELECT id FROM link_banner WHERE pub_id = $1) AND platform = 'Web'"
    )
    .bind(banner.identifier)
    .fetch_one(&pool)
    .await
    .expect("could not query impression");

    assert_eq!(
        1, count,
        "expected one impression to be inserted, got: {count}"
    );
}

#[sqlx::test]
async fn test_banners_v3_impression_post_allows_multiple_impressions(pool: PgPool) {
    let banner = BannerBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "bannerId": banner.identifier,
        "platform": "ios"
    });

    for _ in 0..3 {
        let result = TestClient::new(pool.clone())
            .await
            .authorized()
            .post("/v3/banners/impression", request_body.clone())
            .await;

        assert_eq!(StatusCode::OK, result.status());
    }

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM impression_banner WHERE banner_link = (SELECT id FROM link_banner WHERE pub_id = $1) AND platform = 'IOS'"
    )
    .bind(banner.identifier)
    .fetch_one(&pool)
    .await
    .expect("could not query impression");

    assert_eq!(
        3, count,
        "expected three impressions to be inserted, got: {count}"
    );
}

#[sqlx::test]
async fn test_banners_v3_impression_post_returns_error_for_nonexistent_banner(pool: PgPool) {
    let nonexistent_banner_id = uuid::Uuid::new_v4();

    let request_body = serde_json::json!({
        "bannerId": nonexistent_banner_id,
        "platform": "ios"
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/banners/impression", request_body)
        .await;

    assert!(
        result.status() != StatusCode::OK,
        "expected error status for nonexistent banner, got: {status:?}",
        status = result.status()
    );
}

#[sqlx::test]
async fn test_banners_v3_impression_post_returns_error_for_missing_banner_id(pool: PgPool) {
    let request_body = serde_json::json!({
        "platform": "ios"
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/banners/impression", request_body)
        .await;

    assert!(
        result.status() != StatusCode::OK,
        "expected error status for missing bannerId, got: {status:?}",
        status = result.status()
    );
}

#[sqlx::test]
async fn test_banners_v3_impression_post_returns_error_for_missing_platform(pool: PgPool) {
    let banner = BannerBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "bannerId": banner.identifier
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/banners/impression", request_body)
        .await;

    assert!(
        result.status() != StatusCode::OK,
        "expected error status for missing platform, got: {status:?}",
        status = result.status()
    );
}

#[sqlx::test]
async fn test_banners_v3_impression_post_returns_error_for_invalid_platform(pool: PgPool) {
    let banner = BannerBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "bannerId": banner.identifier,
        "platform": "invalid_platform"
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/banners/impression", request_body)
        .await;

    assert!(
        result.status() != StatusCode::OK,
        "expected error status for invalid platform, got: {status:?}",
        status = result.status()
    );
}
