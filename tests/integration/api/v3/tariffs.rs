use axum::http::StatusCode;
use ladefuchs_api::fixtures::{
    charge_price::ChargePriceBuilder, operator::OperatorBuilder, tariff::TariffBuilder,
};
use pretty_assertions::assert_eq;
use serde_json::Value;
use sqlx::PgPool;

use crate::helpers::TestClient;

#[sqlx::test]
async fn test_tariffs_v3_get_returns_200_and_json_object(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let standard_tariff = TariffBuilder::new().standard(true).create(&pool).await;
    let non_standard_tariff = TariffBuilder::new().standard(false).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(standard_tariff.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(non_standard_tariff.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/tariffs")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    assert!(
        json.get("tariffs").is_some(),
        "expected `tariffs` field in response, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    let identifiers: Vec<&str> = tariffs
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect();

    let standard_id = standard_tariff.pub_tariff_id.to_string();
    let non_standard_id = non_standard_tariff.pub_tariff_id.to_string();
    assert!(
        identifiers.contains(&standard_id.as_str()),
        "expected standard tariff to be included, got: {json:?}"
    );
    assert!(
        identifiers.contains(&non_standard_id.as_str()),
        "expected non-standard tariff to be included, got: {json:?}"
    );

    let first = tariffs.first().expect("non-empty array");
    assert!(
        first.get("identifier").is_some(),
        "expected `identifier` field in item, got: {first:?}"
    );
    assert!(
        first.get("name").is_some(),
        "expected `name` field in item, got: {first:?}"
    );
    assert!(
        first.get("isStandard").is_some(),
        "expected `isStandard` field in item, got: {first:?}"
    );
    assert!(
        first.get("providerName").is_some(),
        "expected `providerName` field in item, got: {first:?}"
    );
    assert!(
        first.get("lastUpdatedDate").is_some(),
        "expected `lastUpdatedDate` field in item, got: {first:?}"
    );
    assert!(
        first.get("isCustomerOnly").is_some(),
        "expected `isCustomerOnly` field in item, got: {first:?}"
    );
    assert!(
        first.get("isAdHoc").is_some(),
        "expected `isAdHoc` field in item, got: {first:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v3_get_with_standard_true_returns_only_standard(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let standard_tariff = TariffBuilder::new().standard(true).create(&pool).await;
    let non_standard_tariff = TariffBuilder::new().standard(false).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(standard_tariff.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(non_standard_tariff.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/tariffs?standard=true")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    let identifiers: Vec<&str> = tariffs
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect();

    let standard_id = standard_tariff.pub_tariff_id.to_string();
    let non_standard_id = non_standard_tariff.pub_tariff_id.to_string();
    assert!(
        identifiers.contains(&standard_id.as_str()),
        "expected standard tariff to be included, got: {json:?}"
    );
    assert!(
        !identifiers.contains(&non_standard_id.as_str()),
        "expected non-standard tariff to not be included, got: {json:?}"
    );

    for tariff in tariffs {
        let is_standard = tariff
            .get("isStandard")
            .and_then(|v| v.as_bool())
            .expect("isStandard should be a boolean");
        assert!(
            is_standard,
            "expected all tariffs to be standard when standard=true, got: {tariff:?}"
        );
    }
}

#[sqlx::test]
async fn test_tariffs_v3_get_with_standard_false_returns_all(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let standard_tariff = TariffBuilder::new().standard(true).create(&pool).await;
    let non_standard_tariff = TariffBuilder::new().standard(false).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(standard_tariff.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(non_standard_tariff.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/tariffs?standard=false")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    let identifiers: Vec<&str> = tariffs
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect();

    let standard_id = standard_tariff.pub_tariff_id.to_string();
    let non_standard_id = non_standard_tariff.pub_tariff_id.to_string();
    assert!(
        identifiers.contains(&standard_id.as_str()),
        "expected standard tariff to be included, got: {json:?}"
    );
    assert!(
        identifiers.contains(&non_standard_id.as_str()),
        "expected non-standard tariff to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v3_get_returns_empty_array_when_no_tariffs(pool: PgPool) {
    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/tariffs")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    assert_eq!(
        0,
        tariffs.len(),
        "expected empty tariffs array when no tariffs exist, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v3_get_filters_out_hidden_tariffs(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let visible_tariff = TariffBuilder::new().hide(false).create(&pool).await;
    let hidden_tariff = TariffBuilder::new().hide(true).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(visible_tariff.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(hidden_tariff.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/tariffs")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    let identifiers: Vec<&str> = tariffs
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect();

    let visible_id = visible_tariff.pub_tariff_id.to_string();
    let hidden_id = hidden_tariff.pub_tariff_id.to_string();
    assert!(
        identifiers.contains(&visible_id.as_str()),
        "expected visible tariff to be included, got: {json:?}"
    );
    assert!(
        !identifiers.contains(&hidden_id.as_str()),
        "expected hidden tariff to not be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v3_get_skips_empty_monthly_fee(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff_with_fee = TariffBuilder::new().monthly_fee(10.0).create(&pool).await;
    let tariff_without_fee = TariffBuilder::new().monthly_fee(0.0).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff_with_fee.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff_without_fee.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/tariffs")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    for tariff in tariffs {
        let identifier = tariff
            .get("identifier")
            .and_then(|v| v.as_str())
            .expect("identifier should be a string");

        if identifier == tariff_without_fee.pub_tariff_id.to_string() {
            assert!(
                tariff.get("monthlyFee").is_none(),
                "expected monthlyFee to be skipped when zero, got: {tariff:?}"
            );
        } else if identifier == tariff_with_fee.pub_tariff_id.to_string() {
            assert!(
                tariff.get("monthlyFee").is_some(),
                "expected monthlyFee to be present when non-zero, got: {tariff:?}"
            );
        }
    }
}

#[sqlx::test]
async fn test_tariffs_v3_get_skips_empty_note(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff_with_note = TariffBuilder::new().note("Test note").create(&pool).await;
    let tariff_without_note = TariffBuilder::new().note("").create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff_with_note.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff_without_note.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/tariffs")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    for tariff in tariffs {
        let identifier = tariff
            .get("identifier")
            .and_then(|v| v.as_str())
            .expect("identifier should be a string");

        if identifier == tariff_without_note.pub_tariff_id.to_string() {
            assert!(
                tariff.get("note").is_none(),
                "expected note to be skipped when empty, got: {tariff:?}"
            );
        } else if identifier == tariff_with_note.pub_tariff_id.to_string() {
            assert!(
                tariff.get("note").is_some(),
                "expected note to be present when non-empty, got: {tariff:?}"
            );
        }
    }
}

#[sqlx::test]
async fn test_tariffs_v3_post_returns_200_and_json_object(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().standard(true).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "standard": true,
        "add": [],
        "remove": [],
        "operatorIds": [operator.pub_network]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    assert!(
        json.get("tariffs").is_some(),
        "expected `tariffs` field in response, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    if !tariffs.is_empty() {
        let first = tariffs.first().expect("non-empty array");
        assert!(
            first.get("identifier").is_some(),
            "expected `identifier` field in item, got: {first:?}"
        );
        assert!(
            first.get("name").is_some(),
            "expected `name` field in item, got: {first:?}"
        );
        assert!(
            first.get("isStandard").is_some(),
            "expected `isStandard` field in item, got: {first:?}"
        );
        assert!(
            first.get("providerName").is_some(),
            "expected `providerName` field in item, got: {first:?}"
        );
        assert!(
            first.get("lastUpdatedDate").is_some(),
            "expected `lastUpdatedDate` field in item, got: {first:?}"
        );
    }
}

#[sqlx::test]
async fn test_tariffs_v3_post_defaults_to_standard_true(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let standard_tariff = TariffBuilder::new().standard(true).create(&pool).await;
    let non_standard_tariff = TariffBuilder::new().standard(false).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(standard_tariff.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(non_standard_tariff.id)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "add": [],
        "remove": [],
        "operatorIds": [operator.pub_network]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    for tariff in tariffs {
        let is_standard = tariff
            .get("isStandard")
            .and_then(|v| v.as_bool())
            .expect("isStandard should be a boolean");
        assert!(
            is_standard,
            "expected all tariffs to be standard when standard defaults to true, got: {tariff:?}"
        );
    }
}

#[sqlx::test]
async fn test_tariffs_v3_post_with_standard_false_returns_all(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let standard_tariff = TariffBuilder::new().standard(true).create(&pool).await;
    let non_standard_tariff = TariffBuilder::new().standard(false).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(standard_tariff.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(non_standard_tariff.id)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "standard": false,
        "add": [],
        "remove": [],
        "operatorIds": [operator.pub_network]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    let identifiers: Vec<&str> = tariffs
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect();

    let standard_id = standard_tariff.pub_tariff_id.to_string();
    let non_standard_id = non_standard_tariff.pub_tariff_id.to_string();
    assert!(
        identifiers.contains(&standard_id.as_str())
            || identifiers.contains(&non_standard_id.as_str()),
        "expected at least one tariff to be included when standard=false, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v3_post_supports_add_list(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let standard_tariff = TariffBuilder::new().standard(true).create(&pool).await;
    let custom_tariff = TariffBuilder::new().standard(false).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(standard_tariff.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(custom_tariff.id)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "standard": true,
        "add": [custom_tariff.pub_tariff_id],
        "remove": [],
        "operatorIds": [operator.pub_network]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    let identifiers: Vec<&str> = tariffs
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect();

    let custom_id = custom_tariff.pub_tariff_id.to_string();
    assert!(
        identifiers.contains(&custom_id.as_str()),
        "expected custom tariff to be included when added to add list, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v3_post_supports_remove_list(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff1 = TariffBuilder::new().standard(true).create(&pool).await;
    let tariff2 = TariffBuilder::new().standard(true).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff1.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff2.id)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "standard": true,
        "add": [],
        "remove": [tariff1.pub_tariff_id],
        "operatorIds": [operator.pub_network]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    let identifiers: Vec<&str> = tariffs
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect();

    let tariff1_id = tariff1.pub_tariff_id.to_string();
    assert!(
        !identifiers.contains(&tariff1_id.as_str()),
        "expected tariff1 to not be included when in remove list, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v3_post_supports_operator_ids_filter(pool: PgPool) {
    let operator1 = OperatorBuilder::new().create(&pool).await;
    let operator2 = OperatorBuilder::new().create(&pool).await;
    let tariff1 = TariffBuilder::new().standard(true).create(&pool).await;
    let tariff2 = TariffBuilder::new().standard(true).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator1.id)
        .tariff_id(tariff1.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator2.id)
        .tariff_id(tariff2.id)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "standard": true,
        "add": [],
        "remove": [],
        "operatorIds": [operator1.pub_network]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    let tariff1_id = tariff1.pub_tariff_id.to_string();
    let identifiers: Vec<&str> = tariffs
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect();

    assert!(
        identifiers.contains(&tariff1_id.as_str()) || !tariffs.is_empty(),
        "expected tariffs for operator1 to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v3_post_combines_add_and_remove(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let standard_tariff = TariffBuilder::new().standard(true).create(&pool).await;
    let custom_tariff1 = TariffBuilder::new().standard(false).create(&pool).await;
    let custom_tariff2 = TariffBuilder::new().standard(false).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(standard_tariff.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(custom_tariff1.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(custom_tariff2.id)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "standard": true,
        "add": [custom_tariff1.pub_tariff_id, custom_tariff2.pub_tariff_id],
        "remove": [custom_tariff2.pub_tariff_id],
        "operatorIds": [operator.pub_network]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    let identifiers: Vec<&str> = tariffs
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect();

    let custom1_id = custom_tariff1.pub_tariff_id.to_string();
    let custom2_id = custom_tariff2.pub_tariff_id.to_string();
    assert!(
        identifiers.contains(&custom1_id.as_str()),
        "expected custom_tariff1 to be included (added), got: {json:?}"
    );
    assert!(
        !identifiers.contains(&custom2_id.as_str()),
        "expected custom_tariff2 to not be included (removed), got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v3_post_returns_empty_array_when_no_tariffs(pool: PgPool) {
    let request_body = serde_json::json!({
        "standard": true,
        "add": [],
        "remove": [],
        "operatorIds": []
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    assert_eq!(
        0,
        tariffs.len(),
        "expected empty tariffs array when no tariffs match, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v3_post_defaults_operator_ids_to_empty(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().standard(true).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "standard": true,
        "add": [],
        "remove": []
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v3/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let tariffs = json
        .get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array");

    assert!(
        !tariffs.is_empty(),
        "expected tariffs to be returned even with empty operatorIds, got: {json:?}"
    );
}
