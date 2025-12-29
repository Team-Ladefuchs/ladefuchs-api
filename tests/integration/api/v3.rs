use axum::http::StatusCode;
use ladefuchs_api::fixtures::{
    charge_price::ChargePriceBuilder, operator::OperatorBuilder, tariff::TariffBuilder,
};
use pretty_assertions::assert_eq;
use serde_json::Value;
use sqlx::PgPool;

use crate::helpers::TestClient;

#[sqlx::test]
async fn test_conditions_v3_post_returns_200_and_json_object(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.42)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network],
        "tariffIds": []
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    assert!(
        json.get("chargingConditions").is_some(),
        "expected `chargingConditions` field in response, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    assert!(
        !charging_conditions.is_empty(),
        "expected non-empty chargingConditions array, got: {json:?}"
    );

    let first = charging_conditions.first().expect("non-empty array");
    assert!(
        first.get("operatorId").is_some(),
        "expected `operatorId` field in item, got: {first:?}"
    );
    assert!(
        first.get("tariffConditions").is_some(),
        "expected `tariffConditions` field in item, got: {first:?}"
    );

    let operator_id = first
        .get("operatorId")
        .and_then(|v| v.as_str())
        .expect("operatorId field should be a string");
    assert_eq!(
        operator.pub_network.to_string(),
        operator_id,
        "expected operator ID to match, got: {first:?}"
    );

    let tariff_conditions = first
        .get("tariffConditions")
        .and_then(|v| v.as_array())
        .expect("tariffConditions should be an array");

    if !tariff_conditions.is_empty() {
        let condition = tariff_conditions
            .first()
            .expect("non-empty tariffConditions");
        assert!(
            condition.get("blockingFeeStart").is_some(),
            "expected `blockingFeeStart` field in condition, got: {condition:?}"
        );
        assert!(
            condition.get("blockingFee").is_some(),
            "expected `blockingFee` field in condition, got: {condition:?}"
        );
        assert!(
            condition.get("chargingMode").is_some(),
            "expected `chargingMode` field in condition, got: {condition:?}"
        );
        assert!(
            condition.get("pricePerKwh").is_some(),
            "expected `pricePerKwh` field in condition, got: {condition:?}"
        );
        assert!(
            condition.get("tariffId").is_some(),
            "expected `tariffId` field in condition, got: {condition:?}"
        );
        assert!(
            condition.get("tariffName").is_some(),
            "expected `tariffName` field in condition, got: {condition:?}"
        );
    }
}

#[sqlx::test]
async fn test_conditions_v3_post_returns_empty_array_when_no_operators(pool: PgPool) {
    let request_body = serde_json::json!({
        "operatorIds": [],
        "tariffIds": []
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    assert_eq!(
        0,
        charging_conditions.len(),
        "expected empty chargingConditions when no operators provided, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_conditions_v3_post_returns_empty_conditions_for_nonexistent_operator(pool: PgPool) {
    let nonexistent_id = uuid::Uuid::new_v4();
    let request_body = serde_json::json!({
        "operatorIds": [nonexistent_id],
        "tariffIds": []
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    assert_eq!(
        1,
        charging_conditions.len(),
        "expected one entry in chargingConditions for nonexistent operator, got: {json:?}"
    );

    let first = charging_conditions.first().expect("non-empty array");
    let operator_id = first
        .get("operatorId")
        .and_then(|v| v.as_str())
        .expect("operatorId should be a string");
    assert_eq!(
        nonexistent_id.to_string(),
        operator_id,
        "expected operator ID to match, got: {first:?}"
    );

    let tariff_conditions = first
        .get("tariffConditions")
        .and_then(|v| v.as_array())
        .expect("tariffConditions should be an array");

    assert_eq!(
        0,
        tariff_conditions.len(),
        "expected empty tariffConditions for nonexistent operator, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_conditions_v3_post_supports_cpos_alias(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.42)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "cpos": [operator.pub_network],
        "tariffIds": []
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    assert!(
        !charging_conditions.is_empty(),
        "expected non-empty chargingConditions when using cpos alias, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_conditions_v3_post_handles_multiple_operators(pool: PgPool) {
    let operator1 = OperatorBuilder::new().create(&pool).await;
    let operator2 = OperatorBuilder::new().create(&pool).await;
    let tariff1 = TariffBuilder::new().create(&pool).await;
    let tariff2 = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator1.id)
        .tariff_id(tariff1.id)
        .price(0.42)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator2.id)
        .tariff_id(tariff2.id)
        .price(0.55)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "operatorIds": [operator1.pub_network, operator2.pub_network],
        "tariffIds": []
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    assert!(
        !charging_conditions.is_empty(),
        "expected at least one operator in response, got: {json:?}"
    );

    let operator_ids: Vec<&str> = charging_conditions
        .iter()
        .filter_map(|v| v.get("operatorId").and_then(|id| id.as_str()))
        .collect();

    let operator1_id = operator1.pub_network.to_string();
    let operator2_id = operator2.pub_network.to_string();
    assert!(
        operator_ids.contains(&operator1_id.as_str()),
        "expected operator1 to be included, got: {json:?}"
    );
    assert!(
        operator_ids.contains(&operator2_id.as_str()),
        "expected operator2 to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_conditions_v3_post_supports_tariff_ids_filter(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff1 = TariffBuilder::new().create(&pool).await;
    let tariff2 = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff1.id)
        .price(0.42)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff2.id)
        .price(0.55)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network],
        "tariffIds": [tariff1.pub_tariff_id]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    assert!(
        !charging_conditions.is_empty(),
        "expected non-empty chargingConditions, got: {json:?}"
    );

    let first = charging_conditions.first().expect("non-empty array");
    let tariff_conditions = first
        .get("tariffConditions")
        .and_then(|v| v.as_array())
        .expect("tariffConditions should be an array");

    for condition in tariff_conditions {
        let tariff_id = condition
            .get("tariffId")
            .and_then(|v| v.as_str())
            .expect("tariffId should be a string");
        assert_eq!(
            tariff1.pub_tariff_id.to_string(),
            tariff_id,
            "expected only tariff1 conditions, got: {condition:?}"
        );
    }
}

#[sqlx::test]
async fn test_conditions_v3_post_supports_tariffs_ids_alias(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.42)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network],
        "tariffsIds": [tariff.pub_tariff_id]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    assert!(
        !charging_conditions.is_empty(),
        "expected non-empty chargingConditions when using tariffsIds alias, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_conditions_v3_post_supports_charging_modes_filter(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.42)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network],
        "tariffIds": [],
        "chargingModes": ["AC"]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    if !charging_conditions.is_empty() {
        let first = charging_conditions.first().expect("non-empty array");
        let tariff_conditions = first
            .get("tariffConditions")
            .and_then(|v| v.as_array())
            .expect("tariffConditions should be an array");

        for condition in tariff_conditions {
            let charging_mode = condition
                .get("chargingMode")
                .and_then(|v| v.as_str())
                .expect("chargingMode should be a string");
            assert_eq!(
                "ac",
                charging_mode.to_lowercase(),
                "expected only AC charging modes, got: {condition:?}"
            );
        }
    }
}

#[sqlx::test]
async fn test_conditions_v3_post_defaults_to_ac_and_dc_modes(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.42)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network],
        "tariffIds": []
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    assert!(
        !charging_conditions.is_empty(),
        "expected non-empty chargingConditions with default modes, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_conditions_v3_post_supports_multiple_tariff_ids(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff1 = TariffBuilder::new().create(&pool).await;
    let tariff2 = TariffBuilder::new().create(&pool).await;
    let tariff3 = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff1.id)
        .price(0.42)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff2.id)
        .price(0.55)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff3.id)
        .price(0.66)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network],
        "tariffIds": [tariff1.pub_tariff_id, tariff2.pub_tariff_id]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    assert!(
        !charging_conditions.is_empty(),
        "expected non-empty chargingConditions, got: {json:?}"
    );

    let first = charging_conditions.first().expect("non-empty array");
    let tariff_conditions = first
        .get("tariffConditions")
        .and_then(|v| v.as_array())
        .expect("tariffConditions should be an array");

    let tariff_ids: Vec<&str> = tariff_conditions
        .iter()
        .filter_map(|c| c.get("tariffId").and_then(|v| v.as_str()))
        .collect();

    let tariff1_id = tariff1.pub_tariff_id.to_string();
    let tariff2_id = tariff2.pub_tariff_id.to_string();
    let tariff3_id = tariff3.pub_tariff_id.to_string();

    assert!(
        tariff_ids.contains(&tariff1_id.as_str()),
        "expected tariff1 to be included, got: {json:?}"
    );
    assert!(
        tariff_ids.contains(&tariff2_id.as_str()),
        "expected tariff2 to be included, got: {json:?}"
    );
    assert!(
        !tariff_ids.contains(&tariff3_id.as_str()),
        "expected tariff3 to not be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_conditions_v3_post_returns_empty_for_nonexistent_tariff_ids(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;
    let nonexistent_tariff_id = uuid::Uuid::new_v4();

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.42)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network],
        "tariffIds": [nonexistent_tariff_id]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    assert_eq!(
        1,
        charging_conditions.len(),
        "expected one entry in chargingConditions, got: {json:?}"
    );

    let first = charging_conditions.first().expect("non-empty array");
    let tariff_conditions = first
        .get("tariffConditions")
        .and_then(|v| v.as_array())
        .expect("tariffConditions should be an array");

    assert_eq!(
        0,
        tariff_conditions.len(),
        "expected empty tariffConditions for nonexistent tariff ID, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_conditions_v3_post_combines_tariff_ids_and_charging_modes(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff1 = TariffBuilder::new().create(&pool).await;
    let tariff2 = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff1.id)
        .price(0.42)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff2.id)
        .price(0.55)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network],
        "tariffIds": [tariff1.pub_tariff_id, tariff2.pub_tariff_id],
        "chargingModes": ["AC"]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    if !charging_conditions.is_empty() {
        let first = charging_conditions.first().expect("non-empty array");
        let tariff_conditions = first
            .get("tariffConditions")
            .and_then(|v| v.as_array())
            .expect("tariffConditions should be an array");

        for condition in tariff_conditions {
            let charging_mode = condition
                .get("chargingMode")
                .and_then(|v| v.as_str())
                .expect("chargingMode should be a string");
            assert_eq!(
                "ac",
                charging_mode.to_lowercase(),
                "expected only AC charging modes when filtering, got: {condition:?}"
            );
        }
    }
}

#[sqlx::test]
async fn test_conditions_v3_post_tariff_filtering_works_across_multiple_operators(pool: PgPool) {
    let operator1 = OperatorBuilder::new().create(&pool).await;
    let operator2 = OperatorBuilder::new().create(&pool).await;
    let tariff1 = TariffBuilder::new().create(&pool).await;
    let tariff2 = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator1.id)
        .tariff_id(tariff1.id)
        .price(0.42)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator1.id)
        .tariff_id(tariff2.id)
        .price(0.55)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator2.id)
        .tariff_id(tariff1.id)
        .price(0.66)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator2.id)
        .tariff_id(tariff2.id)
        .price(0.77)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "operatorIds": [operator1.pub_network, operator2.pub_network],
        "tariffIds": [tariff1.pub_tariff_id]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    assert_eq!(
        2,
        charging_conditions.len(),
        "expected two operators in response, got: {json:?}"
    );

    for condition_entry in charging_conditions {
        let tariff_conditions = condition_entry
            .get("tariffConditions")
            .and_then(|v| v.as_array())
            .expect("tariffConditions should be an array");

        for condition in tariff_conditions {
            let tariff_id = condition
                .get("tariffId")
                .and_then(|v| v.as_str())
                .expect("tariffId should be a string");
            assert_eq!(
                tariff1.pub_tariff_id.to_string(),
                tariff_id,
                "expected only tariff1 conditions across all operators, got: {condition:?}"
            );
        }
    }
}

#[sqlx::test]
async fn test_conditions_v3_post_supports_dc_charging_mode_filter(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.42)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network],
        "tariffIds": [],
        "chargingModes": ["DC"]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    if !charging_conditions.is_empty() {
        let first = charging_conditions.first().expect("non-empty array");
        let tariff_conditions = first
            .get("tariffConditions")
            .and_then(|v| v.as_array())
            .expect("tariffConditions should be an array");

        for condition in tariff_conditions {
            let charging_mode = condition
                .get("chargingMode")
                .and_then(|v| v.as_str())
                .expect("chargingMode should be a string");
            assert_eq!(
                "dc",
                charging_mode.to_lowercase(),
                "expected only DC charging modes, got: {condition:?}"
            );
        }
    }
}

#[sqlx::test]
async fn test_conditions_v3_post_supports_explicit_ac_and_dc_modes(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.42)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network],
        "tariffIds": [],
        "chargingModes": ["AC", "DC"]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    assert!(
        !charging_conditions.is_empty(),
        "expected non-empty chargingConditions when explicitly specifying AC and DC, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_conditions_v3_post_returns_400_for_invalid_charging_mode(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network],
        "tariffIds": [],
        "chargingModes": ["INVALID"]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::BAD_REQUEST, result.status());
}

#[sqlx::test]
async fn test_conditions_v3_post_returns_400_for_missing_operator_ids(pool: PgPool) {
    let request_body = serde_json::json!({
        "tariffIds": []
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::BAD_REQUEST, result.status());
}

#[sqlx::test]
async fn test_conditions_v3_post_returns_400_for_missing_tariff_ids(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;

    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::BAD_REQUEST, result.status());
}

#[sqlx::test]
async fn test_conditions_v3_post_sets_last_updated_date_when_conditions_exist(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.42)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network],
        "tariffIds": [tariff.pub_tariff_id]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let last_updated_date = json.get("lastUpdatedDate");
    assert!(
        last_updated_date.is_some(),
        "expected `lastUpdatedDate` field in response, got: {json:?}"
    );

    if let Some(date) = last_updated_date {
        assert!(
            !date.is_null(),
            "expected `lastUpdatedDate` to be set when conditions exist, got: {json:?}"
        );
        assert!(
            date.is_string(),
            "expected `lastUpdatedDate` to be a string (ISO 8601), got: {date:?}"
        );
    }
}

#[sqlx::test]
async fn test_conditions_v3_post_sets_last_updated_date_to_null_when_no_conditions(pool: PgPool) {
    let nonexistent_id = uuid::Uuid::new_v4();
    let request_body = serde_json::json!({
        "operatorIds": [nonexistent_id],
        "tariffIds": []
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let last_updated_date = json.get("lastUpdatedDate");
    assert!(
        last_updated_date.is_some(),
        "expected `lastUpdatedDate` field in response, got: {json:?}"
    );

    if let Some(date) = last_updated_date {
        assert!(
            date.is_null(),
            "expected `lastUpdatedDate` to be null when no conditions exist, got: {json:?}"
        );
    }
}

#[sqlx::test]
async fn test_conditions_v3_post_returns_empty_when_all_filtered_out(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;
    let other_tariff = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.42)
        .create(&pool)
        .await;

    // Request with a different tariff ID, so all conditions should be filtered out
    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network],
        "tariffIds": [other_tariff.pub_tariff_id]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/conditions", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let charging_conditions = json
        .get("chargingConditions")
        .and_then(|v| v.as_array())
        .expect("chargingConditions should be an array");

    assert_eq!(
        1,
        charging_conditions.len(),
        "expected one operator entry, got: {json:?}"
    );

    let first = charging_conditions.first().expect("non-empty array");
    let tariff_conditions = first
        .get("tariffConditions")
        .and_then(|v| v.as_array())
        .expect("tariffConditions should be an array");

    assert_eq!(
        0,
        tariff_conditions.len(),
        "expected empty tariffConditions when all filtered out, got: {json:?}"
    );
}
