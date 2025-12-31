use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use serde_json::Value;
use sqlx::PgPool;

use crate::helpers::TestClient;

#[sqlx::test]
async fn test_app_metrics_v3_post_returns_200_with_provided_device_id(pool: PgPool) {
    let device_id = uuid::Uuid::new_v4();

    let request_body = serde_json::json!({
        "deviceId": device_id,
        "platform": "ios",
        "version": 1
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/app/metrics", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let returned_device_id = json
        .get("deviceId")
        .and_then(|v| v.as_str())
        .expect("deviceId should be a string");

    assert_eq!(
        device_id.to_string(),
        returned_device_id,
        "expected deviceId to match provided value, got: {returned_device_id:?}"
    );

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM app_metrics WHERE app_id = $1 AND platform = 'IOS' AND version = $2",
    )
    .bind(device_id)
    .bind(1i32)
    .fetch_one(&pool)
    .await
    .expect("could not query app_metrics");

    assert!(
        count > 0,
        "expected at least one metric to be inserted, got: {count}"
    );
}

#[sqlx::test]
async fn test_app_metrics_v3_post_generates_device_id_when_not_provided(pool: PgPool) {
    let request_body = serde_json::json!({
        "platform": "android",
        "version": 2
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/app/metrics", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let device_id_str = json
        .get("deviceId")
        .and_then(|v| v.as_str())
        .expect("deviceId should be a string");

    let device_id = uuid::Uuid::parse_str(device_id_str).expect("deviceId should be a valid UUID");

    assert!(
        !device_id.is_nil(),
        "expected generated deviceId to be non-nil, got: {device_id:?}"
    );

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM app_metrics WHERE app_id = $1 AND platform = 'Android' AND version = $2"
    )
    .bind(device_id)
    .bind(2i32)
    .fetch_one(&pool)
    .await
    .expect("could not query app_metrics");

    assert!(
        count > 0,
        "expected at least one metric to be inserted, got: {count}"
    );
}

#[sqlx::test]
async fn test_app_metrics_v3_post_supports_ios_platform(pool: PgPool) {
    let device_id = uuid::Uuid::new_v4();

    let request_body = serde_json::json!({
        "deviceId": device_id,
        "platform": "ios",
        "version": 10
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/app/metrics", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM app_metrics WHERE app_id = $1 AND platform = 'IOS'",
    )
    .bind(device_id)
    .fetch_one(&pool)
    .await
    .expect("could not query app_metrics");

    assert_eq!(
        1, count,
        "expected one metric to be inserted for iOS platform, got: {count}"
    );
}

#[sqlx::test]
async fn test_app_metrics_v3_post_supports_android_platform(pool: PgPool) {
    let device_id = uuid::Uuid::new_v4();

    let request_body = serde_json::json!({
        "deviceId": device_id,
        "platform": "android",
        "version": 20
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/app/metrics", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM app_metrics WHERE app_id = $1 AND platform = 'Android'",
    )
    .bind(device_id)
    .fetch_one(&pool)
    .await
    .expect("could not query app_metrics");

    assert_eq!(
        1, count,
        "expected one metric to be inserted for Android platform, got: {count}"
    );
}

#[sqlx::test]
async fn test_app_metrics_v3_post_supports_web_platform(pool: PgPool) {
    let device_id = uuid::Uuid::new_v4();

    let request_body = serde_json::json!({
        "deviceId": device_id,
        "platform": "web",
        "version": 30
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/app/metrics", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM app_metrics WHERE app_id = $1 AND platform = 'Web'",
    )
    .bind(device_id)
    .fetch_one(&pool)
    .await
    .expect("could not query app_metrics");

    assert_eq!(
        1, count,
        "expected one metric to be inserted for Web platform, got: {count}"
    );
}

#[sqlx::test]
async fn test_app_metrics_v3_post_allows_multiple_entries_for_same_device(pool: PgPool) {
    let device_id = uuid::Uuid::new_v4();

    for version in 1..=3 {
        let request_body = serde_json::json!({
            "deviceId": device_id,
            "platform": "ios",
            "version": version
        });

        let result = TestClient::new(pool.clone())
            .await
            .authorized()
            .post("/v3/app/metrics", request_body)
            .await;

        assert_eq!(StatusCode::OK, result.status());
    }

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM app_metrics WHERE app_id = $1")
        .bind(device_id)
        .fetch_one(&pool)
        .await
        .expect("could not query app_metrics");

    assert_eq!(
        3, count,
        "expected three metrics to be inserted for same device, got: {count}"
    );
}

#[sqlx::test]
async fn test_app_metrics_v3_post_returns_error_for_missing_platform(pool: PgPool) {
    let device_id = uuid::Uuid::new_v4();

    let request_body = serde_json::json!({
        "deviceId": device_id,
        "version": 1
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/app/metrics", request_body)
        .await;

    assert!(
        result.status() != StatusCode::OK,
        "expected error status for missing platform, got: {status:?}",
        status = result.status()
    );
}

#[sqlx::test]
async fn test_app_metrics_v3_post_returns_error_for_missing_version(pool: PgPool) {
    let device_id = uuid::Uuid::new_v4();

    let request_body = serde_json::json!({
        "deviceId": device_id,
        "platform": "ios"
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/app/metrics", request_body)
        .await;

    assert!(
        result.status() != StatusCode::OK,
        "expected error status for missing version, got: {status:?}",
        status = result.status()
    );
}

#[sqlx::test]
async fn test_app_metrics_v3_post_returns_error_for_invalid_platform(pool: PgPool) {
    let device_id = uuid::Uuid::new_v4();

    let request_body = serde_json::json!({
        "deviceId": device_id,
        "platform": "invalid_platform",
        "version": 1
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/app/metrics", request_body)
        .await;

    assert!(
        result.status() != StatusCode::OK,
        "expected error status for invalid platform, got: {status:?}",
        status = result.status()
    );
}

#[sqlx::test]
async fn test_app_metrics_v3_post_handles_zero_version(pool: PgPool) {
    let device_id = uuid::Uuid::new_v4();

    let request_body = serde_json::json!({
        "deviceId": device_id,
        "platform": "ios",
        "version": 0
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/app/metrics", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM app_metrics WHERE app_id = $1 AND version = $2")
            .bind(device_id)
            .bind(0i32)
            .fetch_one(&pool)
            .await
            .expect("could not query app_metrics");

    assert_eq!(
        1, count,
        "expected one metric to be inserted with version 0, got: {count}"
    );
}

#[sqlx::test]
async fn test_app_metrics_v3_post_handles_large_version_number(pool: PgPool) {
    let device_id = uuid::Uuid::new_v4();

    let request_body = serde_json::json!({
        "deviceId": device_id,
        "platform": "ios",
        "version": 65535
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/app/metrics", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM app_metrics WHERE app_id = $1 AND version = $2")
            .bind(device_id)
            .bind(65535i32)
            .fetch_one(&pool)
            .await
            .expect("could not query app_metrics");

    assert_eq!(
        1, count,
        "expected one metric to be inserted with version 65535, got: {count}"
    );
}
