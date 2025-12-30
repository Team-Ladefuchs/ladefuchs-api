use axum::http::StatusCode;
use ladefuchs_api::fixtures::announcement::AnnouncementBuilder;
use pretty_assertions::assert_eq;
use serde_json::Value;
use sqlx::PgPool;

use crate::helpers::TestClient;

#[sqlx::test]
async fn test_announcement_v3_get_returns_200_and_json_value_when_exists(pool: PgPool) {
    let announcement = AnnouncementBuilder::new()
        .value(serde_json::json!({
            "title": "Test Announcement",
            "message": "This is a test announcement",
            "type": "info"
        }))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/announcement")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    assert_eq!(
        announcement.value.get("title"),
        json.get("title"),
        "expected announcement title to match, got: {json:?}"
    );
    assert_eq!(
        announcement.value.get("message"),
        json.get("message"),
        "expected announcement message to match, got: {json:?}"
    );
    assert_eq!(
        announcement.value.get("type"),
        json.get("type"),
        "expected announcement type to match, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_announcement_v3_get_returns_null_when_no_announcement_exists(pool: PgPool) {
    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/announcement")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_null(),
        "expected null response when no announcement exists, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_announcement_v3_get_returns_first_announcement_when_multiple_exist(pool: PgPool) {
    let first_announcement = AnnouncementBuilder::new()
        .value(serde_json::json!({
            "title": "First Announcement",
            "message": "This is the first announcement"
        }))
        .create(&pool)
        .await;

    let _second_announcement = AnnouncementBuilder::new()
        .value(serde_json::json!({
            "title": "Second Announcement",
            "message": "This is the second announcement"
        }))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/announcement")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    assert_eq!(
        first_announcement.value.get("title"),
        json.get("title"),
        "expected first announcement to be returned, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_announcement_v3_get_handles_complex_json_structure(pool: PgPool) {
    AnnouncementBuilder::new()
        .value(serde_json::json!({
            "title": "Complex Announcement",
            "content": {
                "sections": [
                    {
                        "heading": "Section 1",
                        "text": "Content for section 1"
                    },
                    {
                        "heading": "Section 2",
                        "text": "Content for section 2"
                    }
                ]
            },
            "metadata": {
                "priority": "high",
                "category": "update"
            },
            "links": [
                {

                    "text": "Learn more"
                }
            ]
        }))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/announcement")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    assert!(
        json.get("content").is_some(),
        "expected content field in complex announcement, got: {json:?}"
    );
    assert!(
        json.get("metadata").is_some(),
        "expected metadata field in complex announcement, got: {json:?}"
    );
    assert!(
        json.get("links").is_some(),
        "expected links field in complex announcement, got: {json:?}"
    );

    let content = json
        .get("content")
        .and_then(|v| v.as_object())
        .expect("content should be object");
    assert!(
        content.get("sections").is_some(),
        "expected sections in content, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_announcement_v3_get_handles_minimal_json(pool: PgPool) {
    let minimal_announcement = AnnouncementBuilder::new()
        .value(serde_json::json!({
            "text": "Simple announcement"
        }))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/announcement")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    assert_eq!(
        minimal_announcement.value.get("text"),
        json.get("text"),
        "expected announcement text to match, got: {json:?}"
    );
}
