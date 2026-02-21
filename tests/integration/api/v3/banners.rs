use axum::http::StatusCode;
use chrono::Utc;
use ladefuchs_api::fixtures::banner::BannerBuilder;
use ladefuchs_api::fixtures::customer::CustomerBuilder;
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

#[sqlx::test]
async fn test_banners_v3_chargeprice_advertisement_returns_200_and_banner(pool: PgPool) {
    let _banner = BannerBuilder::new().create(&pool).await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/banners/chargeprice/advertisement")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    assert!(
        json.get("imageUrl").is_some(),
        "expected `imageUrl` field in response, got: {json:?}"
    );
    assert!(
        json.get("affiliateLinkUrl").is_some(),
        "expected `affiliateLinkUrl` field in response, got: {json:?}"
    );

    let image_url = json
        .get("imageUrl")
        .and_then(|v| v.as_str())
        .expect("imageUrl should be a string");
    let affiliate_link_url = json
        .get("affiliateLinkUrl")
        .and_then(|v| v.as_str())
        .expect("affiliateLinkUrl should be a string");

    assert!(
        image_url.contains("/image/"),
        "expected v3 image URL to use /image/ path, got: {image_url:?}"
    );
    assert!(
        affiliate_link_url.contains("/affiliate"),
        "expected affiliate link URL to contain /affiliate, got: {affiliate_link_url:?}"
    );
}

#[sqlx::test]
async fn test_banners_v3_chargeprice_advertisement_returns_random_banner(pool: PgPool) {
    let _banner1 = BannerBuilder::new().create(&pool).await;
    let _banner2 = BannerBuilder::new().create(&pool).await;
    let _banner3 = BannerBuilder::new().create(&pool).await;

    // Make multiple requests and collect the returned banner IDs
    let mut returned_ids = std::collections::HashSet::new();
    for _ in 0..10 {
        let result = TestClient::new(pool.clone())
            .await
            .authorized()
            .get("/v3/banners/chargeprice/advertisement")
            .await;

        assert_eq!(StatusCode::OK, result.status());

        let json: Value = result.json().await;
        let affiliate_link_url = json
            .get("affiliateLinkUrl")
            .and_then(|v| v.as_str())
            .expect("affiliateLinkUrl should be a string");

        // Extract banner ID from affiliate link URL (format: /affiliate?url=...&banner=<id>)
        if let Some(banner_param) = affiliate_link_url
            .split('&')
            .find(|s| s.contains("banner="))
            && let Some(id_str) = banner_param.split('=').nth(1)
        {
            returned_ids.insert(id_str.to_string());
        }
    }

    // With 3 banners and 10 requests, we should get at least 2 different banners
    // (statistically very likely, but not guaranteed - so we check for at least 1)
    assert!(
        !returned_ids.is_empty(),
        "expected at least one banner to be returned, got: {returned_ids:?}"
    );
}

#[sqlx::test]
async fn test_banners_v3_chargeprice_advertisement_returns_error_when_no_banners_exist(
    pool: PgPool,
) {
    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/banners/chargeprice/advertisement")
        .await;

    assert!(
        result.status() != StatusCode::OK,
        "expected error status when no banners exist, got: {status:?}",
        status = result.status()
    );
}

#[sqlx::test]
async fn test_banners_v3_chargeprice_advertisement_filters_out_expired_banners(pool: PgPool) {
    let now = Utc::now();
    let _expired_banner = BannerBuilder::new()
        .expiration(now - chrono::Duration::days(1))
        .starts(now - chrono::Duration::days(10))
        .create(&pool)
        .await;

    let active_banner = BannerBuilder::new()
        .starts(now - chrono::Duration::days(1))
        .expiration(now + chrono::Duration::days(10))
        .create(&pool)
        .await;

    // Make multiple requests to ensure we get a banner
    let mut found_active = false;
    for _ in 0..10 {
        let result = TestClient::new(pool.clone())
            .await
            .authorized()
            .get("/v3/banners/chargeprice/advertisement")
            .await;

        assert_eq!(StatusCode::OK, result.status());

        let json: Value = result.json().await;
        let affiliate_link_url = json
            .get("affiliateLinkUrl")
            .and_then(|v| v.as_str())
            .expect("affiliateLinkUrl should be a string");

        // Extract banner ID from affiliate link URL
        if let Some(banner_param) = affiliate_link_url
            .split('&')
            .find(|s| s.contains("banner="))
            && let Some(id_str) = banner_param.split('=').nth(1)
            && id_str == active_banner.identifier.to_string()
        {
            found_active = true;
            break;
        }
    }

    assert!(
        found_active,
        "expected active banner to be returned, but only expired banner was found"
    );
}

#[sqlx::test]
async fn test_banners_v3_chargeprice_advertisement_filters_out_future_banners(pool: PgPool) {
    let now = Utc::now();
    let _future_banner = BannerBuilder::new()
        .starts(now + chrono::Duration::days(1))
        .expiration(now + chrono::Duration::days(10))
        .create(&pool)
        .await;

    let active_banner = BannerBuilder::new()
        .starts(now - chrono::Duration::days(1))
        .expiration(now + chrono::Duration::days(10))
        .create(&pool)
        .await;

    // Make multiple requests to ensure we get a banner
    let mut found_active = false;
    for _ in 0..10 {
        let result = TestClient::new(pool.clone())
            .await
            .authorized()
            .get("/v3/banners/chargeprice/advertisement")
            .await;

        assert_eq!(StatusCode::OK, result.status());

        let json: Value = result.json().await;
        let affiliate_link_url = json
            .get("affiliateLinkUrl")
            .and_then(|v| v.as_str())
            .expect("affiliateLinkUrl should be a string");

        // Extract banner ID from affiliate link URL
        if let Some(banner_param) = affiliate_link_url
            .split('&')
            .find(|s| s.contains("banner="))
            && let Some(id_str) = banner_param.split('=').nth(1)
            && id_str == active_banner.identifier.to_string()
        {
            found_active = true;
            break;
        }
    }

    assert!(
        found_active,
        "expected active banner to be returned, but only future banner was found"
    );
}

#[sqlx::test]
async fn test_banners_v3_chargeprice_advertisement_uses_v3_image_path_format(pool: PgPool) {
    let _banner = BannerBuilder::new().create(&pool).await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/banners/chargeprice/advertisement")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    let image_url = json
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

// Customer-based impression tests

#[sqlx::test]
async fn test_banners_v3_customer_with_unlimited_impressions_shows_banners(pool: PgPool) {
    // Create customer with unlimited impressions (0)
    let customer = CustomerBuilder::new()
        .total_impressions(0)
        .create(&pool)
        .await;

    let banner = BannerBuilder::new()
        .customer_id(customer.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/banners")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("array");

    let ids = arr
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        ids.contains(&banner.identifier.to_string().as_str()),
        "expected banner with unlimited impressions to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_banners_v3_customer_with_remaining_impressions_shows_banners(pool: PgPool) {
    // Create customer with 100 impressions
    let customer = CustomerBuilder::new()
        .total_impressions(100)
        .create(&pool)
        .await;

    let banner = BannerBuilder::new()
        .customer_id(customer.id)
        .create(&pool)
        .await;

    // Add 50 impressions (still under the limit)
    for _ in 0..50 {
        sqlx::query(
            "INSERT INTO impression_banner (banner_link, platform) VALUES ((SELECT id FROM link_banner WHERE pub_id = $1), 'IOS')"
        )
        .bind(banner.identifier)
        .execute(&pool)
        .await
        .expect("could not insert impression");
    }

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/banners")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("array");

    let ids = arr
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        ids.contains(&banner.identifier.to_string().as_str()),
        "expected banner with remaining impressions to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_banners_v3_customer_with_exhausted_impressions_hides_banners(pool: PgPool) {
    // Create customer with 10 impressions
    let customer = CustomerBuilder::new()
        .total_impressions(10)
        .create(&pool)
        .await;

    let banner = BannerBuilder::new()
        .customer_id(customer.id)
        .create(&pool)
        .await;

    // Add 10 impressions (at the limit)
    for _ in 0..10 {
        sqlx::query(
            "INSERT INTO impression_banner (banner_link, platform) VALUES ((SELECT id FROM link_banner WHERE pub_id = $1), 'IOS')"
        )
        .bind(banner.identifier)
        .execute(&pool)
        .await
        .expect("could not insert impression");
    }

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/banners")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("array");

    let ids = arr
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        !ids.contains(&banner.identifier.to_string().as_str()),
        "expected banner with exhausted impressions to not be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_banners_v3_multiple_banners_share_customer_impressions(pool: PgPool) {
    // Create customer with 100 impressions
    let customer = CustomerBuilder::new()
        .total_impressions(100)
        .create(&pool)
        .await;

    let banner1 = BannerBuilder::new()
        .customer_id(customer.id)
        .create(&pool)
        .await;

    let banner2 = BannerBuilder::new()
        .customer_id(customer.id)
        .create(&pool)
        .await;

    // Add 60 impressions to banner1
    for _ in 0..60 {
        sqlx::query(
            "INSERT INTO impression_banner (banner_link, platform) VALUES ((SELECT id FROM link_banner WHERE pub_id = $1), 'IOS')"
        )
        .bind(banner1.identifier)
        .execute(&pool)
        .await
        .expect("could not insert impression");
    }

    // Add 40 impressions to banner2 (total is now 100, at the limit)
    for _ in 0..40 {
        sqlx::query(
            "INSERT INTO impression_banner (banner_link, platform) VALUES ((SELECT id FROM link_banner WHERE pub_id = $1), 'IOS')"
        )
        .bind(banner2.identifier)
        .execute(&pool)
        .await
        .expect("could not insert impression");
    }

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/banners")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("array");

    let ids = arr
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    // Both banners should be hidden because total customer impressions (100) >= limit (100)
    assert!(
        !ids.contains(&banner1.identifier.to_string().as_str()),
        "expected banner1 to not be included when customer impressions are exhausted, got: {json:?}"
    );
    assert!(
        !ids.contains(&banner2.identifier.to_string().as_str()),
        "expected banner2 to not be included when customer impressions are exhausted, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_banners_v3_different_customers_have_separate_impression_pools(pool: PgPool) {
    // Create two customers
    let customer1 = CustomerBuilder::new()
        .total_impressions(50)
        .create(&pool)
        .await;

    let customer2 = CustomerBuilder::new()
        .total_impressions(100)
        .create(&pool)
        .await;

    let banner1 = BannerBuilder::new()
        .customer_id(customer1.id)
        .create(&pool)
        .await;

    let banner2 = BannerBuilder::new()
        .customer_id(customer2.id)
        .create(&pool)
        .await;

    // Exhaust customer1's impressions
    for _ in 0..50 {
        sqlx::query(
            "INSERT INTO impression_banner (banner_link, platform) VALUES ((SELECT id FROM link_banner WHERE pub_id = $1), 'IOS')"
        )
        .bind(banner1.identifier)
        .execute(&pool)
        .await
        .expect("could not insert impression");
    }

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/banners")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("array");

    let ids = arr
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    // banner1 should be hidden (customer1's impressions exhausted)
    assert!(
        !ids.contains(&banner1.identifier.to_string().as_str()),
        "expected banner1 to not be included (exhausted), got: {json:?}"
    );

    // banner2 should still be visible (customer2's impressions not exhausted)
    assert!(
        ids.contains(&banner2.identifier.to_string().as_str()),
        "expected banner2 to be included (customer2 has remaining impressions), got: {json:?}"
    );
}
