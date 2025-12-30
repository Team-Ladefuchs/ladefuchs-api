use axum::http::StatusCode;
use ladefuchs_api::fixtures::{operator::OperatorBuilder, tariff::TariffBuilder};
use pretty_assertions::assert_eq;
use serde_json::Value;
use sqlx::PgPool;

use crate::helpers::TestClient;

#[sqlx::test]
async fn test_feedback_v3_post_wrong_price_returns_200(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "context": {
            "operatorId": operator.pub_network,
            "tariffId": tariff.pub_tariff_id,
            "language": "de"
        },
        "request": {
            "type": "wrongPriceFeedback",
            "attributes": {
                "notes": "The displayed price was incorrect. I paid more than shown.",
                "displayedPrice": 0.45,
                "actualPrice": 0.50,
                "chargeType": "ac"
            }
        }
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/feedback", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM feedback WHERE operator_id = $1 AND tariff_id = $2",
    )
    .bind(operator.id)
    .bind(tariff.id)
    .fetch_one(&pool)
    .await
    .expect("could not query feedback");

    assert_eq!(
        1, count,
        "expected one feedback to be inserted, got: {count}"
    );
}

#[sqlx::test]
async fn test_feedback_v3_post_other_feedback_returns_200(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "context": {
            "operatorId": operator.pub_network,
            "tariffId": tariff.pub_tariff_id
        },
        "request": {
            "type": "otherFeedback",
            "attributes": {
                "notes": "This is some other feedback about the service."
            }
        }
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/feedback", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM feedback WHERE operator_id = $1 AND tariff_id = $2 AND kind = 'other'"
    )
    .bind(operator.id)
    .bind(tariff.id)
    .fetch_one(&pool)
    .await
    .expect("could not query feedback");

    assert_eq!(
        1, count,
        "expected one feedback to be inserted, got: {count}"
    );
}

#[sqlx::test]
async fn test_feedback_v3_post_wrong_price_with_dc_charge_type(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "context": {
            "operatorId": operator.pub_network,
            "tariffId": tariff.pub_tariff_id
        },
        "request": {
            "type": "wrongPriceFeedback",
            "attributes": {
                "notes": "DC charging price was wrong at the station.",
                "displayedPrice": 0.60,
                "actualPrice": 0.65,
                "chargeType": "dc"
            }
        }
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/feedback", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let context: Option<Value> = sqlx::query_scalar(
        "SELECT context FROM feedback WHERE operator_id = $1 AND tariff_id = $2",
    )
    .bind(operator.id)
    .bind(tariff.id)
    .fetch_optional(&pool)
    .await
    .expect("could not query feedback");

    assert!(
        context.is_some(),
        "expected feedback context to be present, got: {context:?}"
    );

    let context_value = context.unwrap();
    let context_obj = context_value.as_object().expect("context should be object");
    assert_eq!(
        context_obj.get("charge_type").and_then(|v| v.as_str()),
        Some("dc"),
        "expected charge_type to be 'dc', got: {context_obj:?}"
    );
}

#[sqlx::test]
async fn test_feedback_v3_post_wrong_price_without_charge_type(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "context": {
            "operatorId": operator.pub_network,
            "tariffId": tariff.pub_tariff_id
        },
        "request": {
            "type": "wrongPriceFeedback",
            "attributes": {
                "notes": "Price was incorrect but I don't know the charge type.",
                "displayedPrice": 0.40,
                "actualPrice": 0.45
            }
        }
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/feedback", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM feedback WHERE operator_id = $1 AND tariff_id = $2",
    )
    .bind(operator.id)
    .bind(tariff.id)
    .fetch_one(&pool)
    .await
    .expect("could not query feedback");

    assert_eq!(
        1, count,
        "expected one feedback to be inserted, got: {count}"
    );
}

#[sqlx::test]
async fn test_feedback_v3_post_defaults_language_to_de(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "context": {
            "operatorId": operator.pub_network,
            "tariffId": tariff.pub_tariff_id
        },
        "request": {
            "type": "otherFeedback",
            "attributes": {
                "notes": "This feedback has no language specified."
            }
        }
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/feedback", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let language: String = sqlx::query_scalar(
        "SELECT language FROM feedback WHERE operator_id = $1 AND tariff_id = $2",
    )
    .bind(operator.id)
    .bind(tariff.id)
    .fetch_one(&pool)
    .await
    .expect("could not query feedback");

    assert_eq!(
        "de", language,
        "expected language to default to 'de', got: {language:?}"
    );
}

#[sqlx::test]
async fn test_feedback_v3_post_rejects_short_notes_for_wrong_price(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "context": {
            "operatorId": operator.pub_network,
            "tariffId": tariff.pub_tariff_id
        },
        "request": {
            "type": "wrongPriceFeedback",
            "attributes": {
                "notes": "Too short",
                "displayedPrice": 0.45,
                "actualPrice": 0.50
            }
        }
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/feedback", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM feedback WHERE operator_id = $1 AND tariff_id = $2",
    )
    .bind(operator.id)
    .bind(tariff.id)
    .fetch_one(&pool)
    .await
    .expect("could not query feedback");

    assert_eq!(
        0, count,
        "expected no feedback to be inserted when notes are too short, got: {count}"
    );
}

#[sqlx::test]
async fn test_feedback_v3_post_rejects_short_notes_for_other_feedback(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "context": {
            "operatorId": operator.pub_network,
            "tariffId": tariff.pub_tariff_id
        },
        "request": {
            "type": "otherFeedback",
            "attributes": {
                "notes": "Too short"
            }
        }
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/feedback", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM feedback WHERE operator_id = $1 AND tariff_id = $2",
    )
    .bind(operator.id)
    .bind(tariff.id)
    .fetch_one(&pool)
    .await
    .expect("could not query feedback");

    assert_eq!(
        0, count,
        "expected no feedback to be inserted when notes are too short, got: {count}"
    );
}

#[sqlx::test]
async fn test_feedback_v3_post_rejects_wrong_price_when_prices_match(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "context": {
            "operatorId": operator.pub_network,
            "tariffId": tariff.pub_tariff_id
        },
        "request": {
            "type": "wrongPriceFeedback",
            "attributes": {
                "notes": "The prices are the same, so this should be rejected.",
                "displayedPrice": 0.45,
                "actualPrice": 0.45
            }
        }
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/feedback", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM feedback WHERE operator_id = $1 AND tariff_id = $2",
    )
    .bind(operator.id)
    .bind(tariff.id)
    .fetch_one(&pool)
    .await
    .expect("could not query feedback");

    assert_eq!(
        0, count,
        "expected no feedback to be inserted when prices match, got: {count}"
    );
}

#[sqlx::test]
async fn test_feedback_v3_post_returns_error_for_nonexistent_operator(pool: PgPool) {
    let nonexistent_operator_id = uuid::Uuid::new_v4();
    let tariff = TariffBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "context": {
            "operatorId": nonexistent_operator_id,
            "tariffId": tariff.pub_tariff_id
        },
        "request": {
            "type": "otherFeedback",
            "attributes": {
                "notes": "This operator does not exist in the database."
            }
        }
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/feedback", request_body)
        .await;

    assert!(
        result.status() != StatusCode::OK,
        "expected error status for nonexistent operator, got: {status:?}",
        status = result.status()
    );
}

#[sqlx::test]
async fn test_feedback_v3_post_returns_error_for_nonexistent_tariff(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let nonexistent_tariff_id = uuid::Uuid::new_v4();

    let request_body = serde_json::json!({
        "context": {
            "operatorId": operator.pub_network,
            "tariffId": nonexistent_tariff_id
        },
        "request": {
            "type": "otherFeedback",
            "attributes": {
                "notes": "This tariff does not exist in the database."
            }
        }
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/feedback", request_body)
        .await;

    assert!(
        result.status() != StatusCode::OK,
        "expected error status for nonexistent tariff, got: {status:?}",
        status = result.status()
    );
}

#[sqlx::test]
async fn test_feedback_v3_post_returns_error_for_missing_context(pool: PgPool) {
    let request_body = serde_json::json!({
        "request": {
            "type": "otherFeedback",
            "attributes": {
                "notes": "Missing context in the request."
            }
        }
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/feedback", request_body)
        .await;

    assert!(
        result.status() != StatusCode::OK,
        "expected error status for missing context, got: {status:?}",
        status = result.status()
    );
}

#[sqlx::test]
async fn test_feedback_v3_post_returns_error_for_missing_request(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "context": {
            "operatorId": operator.pub_network,
            "tariffId": tariff.pub_tariff_id
        }
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/feedback", request_body)
        .await;

    assert!(
        result.status() != StatusCode::OK,
        "expected error status for missing request, got: {status:?}",
        status = result.status()
    );
}

#[sqlx::test]
async fn test_feedback_v3_post_stores_context_for_wrong_price_feedback(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "context": {
            "operatorId": operator.pub_network,
            "tariffId": tariff.pub_tariff_id
        },
        "request": {
            "type": "wrongPriceFeedback",
            "attributes": {
                "notes": "The price context should be stored in the database.",
                "displayedPrice": 0.35,
                "actualPrice": 0.40,
                "chargeType": "ac"
            }
        }
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/feedback", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let context: Option<Value> = sqlx::query_scalar(
        "SELECT context FROM feedback WHERE operator_id = $1 AND tariff_id = $2",
    )
    .bind(operator.id)
    .bind(tariff.id)
    .fetch_optional(&pool)
    .await
    .expect("could not query feedback");

    assert!(
        context.is_some(),
        "expected feedback context to be present, got: {context:?}"
    );

    let context_value = context.unwrap();
    let context_obj = context_value.as_object().expect("context should be object");
    assert!(
        context_obj.get("displayed_price").is_some(),
        "expected displayed_price in context, got: {context_obj:?}"
    );
    assert!(
        context_obj.get("actual_price").is_some(),
        "expected actual_price in context, got: {context_obj:?}"
    );

    let displayed_price = context_obj
        .get("displayed_price")
        .and_then(|v| v.as_f64())
        .expect("displayed_price should be a number");
    let actual_price = context_obj
        .get("actual_price")
        .and_then(|v| v.as_f64())
        .expect("actual_price should be a number");
    assert!(
        (displayed_price - 0.35).abs() < 0.01,
        "expected displayed_price to be close to 0.35, got: {displayed_price}"
    );
    assert!(
        (actual_price - 0.40).abs() < 0.01,
        "expected actual_price to be close to 0.40, got: {actual_price}"
    );
}

#[sqlx::test]
async fn test_feedback_v3_post_stores_no_context_for_other_feedback(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "context": {
            "operatorId": operator.pub_network,
            "tariffId": tariff.pub_tariff_id
        },
        "request": {
            "type": "otherFeedback",
            "attributes": {
                "notes": "Other feedback should not have context stored."
            }
        }
    });

    let result = TestClient::new(pool.clone())
        .await
        .authorized()
        .post("/v3/feedback", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let context: Option<Value> = sqlx::query_scalar(
        "SELECT context FROM feedback WHERE operator_id = $1 AND tariff_id = $2",
    )
    .bind(operator.id)
    .bind(tariff.id)
    .fetch_optional(&pool)
    .await
    .expect("could not query feedback");

    let is_null = context.as_ref().map(|v| v.is_null()).unwrap_or(true);
    assert!(
        context.is_none() || is_null,
        "expected context to be null for other feedback, got: {context:?}"
    );
}
