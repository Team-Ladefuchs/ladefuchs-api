use axum::http::StatusCode;
use ladefuchs_api::fixtures::{
    charge_price::ChargePriceBuilder, dynamic_price::DynamicChargePriceBuilder,
    operator::OperatorBuilder, tariff::TariffBuilder,
};
use ladefuchs_api::ladefuchs_db::plug::ChargeType;
use pretty_assertions::assert_eq;
use serde_json::Value;
use sqlx::PgPool;

use crate::helpers::TestClient;

fn tariffs_of(json: &Value) -> &Vec<Value> {
    json.get("tariffs")
        .and_then(|v| v.as_array())
        .expect("tariffs should be an array")
}

fn identifiers(json: &Value) -> Vec<String> {
    tariffs_of(json)
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .map(|s| s.to_owned())
        .collect()
}

fn find_tariff<'a>(json: &'a Value, pub_tariff_id: &uuid::Uuid) -> &'a Value {
    tariffs_of(json)
        .iter()
        .find(|v| {
            v.get("identifier").and_then(|id| id.as_str())
                == Some(pub_tariff_id.to_string().as_str())
        })
        .unwrap_or_else(|| panic!("tariff {pub_tariff_id} should be present, got: {json:?}"))
}

#[sqlx::test]
async fn test_tariffs_v4_get_returns_200_and_json_object(pool: PgPool) {
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
        .get("/v4/tariffs")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let ids = identifiers(&json);
    assert!(
        ids.contains(&standard_tariff.pub_tariff_id.to_string()),
        "expected standard tariff to be included, got: {json:?}"
    );
    assert!(
        ids.contains(&non_standard_tariff.pub_tariff_id.to_string()),
        "expected non-standard tariff to be included, got: {json:?}"
    );

    let first = tariffs_of(&json).first().expect("non-empty array");
    for field in [
        "identifier",
        "name",
        "isStandard",
        "providerName",
        "lastUpdatedDate",
        "isCustomerOnly",
        "isAdHoc",
        "isDynamic",
    ] {
        assert!(
            first.get(field).is_some(),
            "expected `{field}` field in item, got: {first:?}"
        );
    }
}

#[sqlx::test]
async fn test_tariffs_v4_get_with_standard_true_returns_only_standard(pool: PgPool) {
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
        .get("/v4/tariffs?standard=true")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let ids = identifiers(&json);

    assert!(
        ids.contains(&standard_tariff.pub_tariff_id.to_string()),
        "expected standard tariff to be included, got: {json:?}"
    );
    assert!(
        !ids.contains(&non_standard_tariff.pub_tariff_id.to_string()),
        "expected non-standard tariff to be excluded, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_is_dynamic_false_for_regular_charge_price(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().standard(true).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v4/tariffs")
        .await;

    let json: Value = result.json().await;

    let item = tariffs_of(&json)
        .iter()
        .find(|v| {
            v.get("identifier").and_then(|id| id.as_str())
                == Some(tariff.pub_tariff_id.to_string().as_str())
        })
        .expect("tariff should be present");

    assert_eq!(
        Some(false),
        item.get("isDynamic").and_then(|v| v.as_bool()),
        "expected isDynamic == false for a charge-price-only tariff, got: {item:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_is_dynamic_true_for_dynamic_price(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().standard(true).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .create(&pool)
        .await;
    DynamicChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v4/tariffs")
        .await;

    let json: Value = result.json().await;

    let item = tariffs_of(&json)
        .iter()
        .find(|v| {
            v.get("identifier").and_then(|id| id.as_str())
                == Some(tariff.pub_tariff_id.to_string().as_str())
        })
        .expect("tariff should be present");

    assert_eq!(
        Some(true),
        item.get("isDynamic").and_then(|v| v.as_bool()),
        "expected isDynamic == true for a tariff with dynamic prices, got: {item:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_includes_dynamic_only_msp(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let dynamic_only = TariffBuilder::new().standard(false).create(&pool).await;

    DynamicChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(dynamic_only.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v4/tariffs")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let ids = identifiers(&json);

    assert!(
        ids.contains(&dynamic_only.pub_tariff_id.to_string()),
        "expected a dynamic-only MSP tariff to be included in v4, got: {json:?}"
    );

    let item = tariffs_of(&json)
        .iter()
        .find(|v| {
            v.get("identifier").and_then(|id| id.as_str())
                == Some(dynamic_only.pub_tariff_id.to_string().as_str())
        })
        .expect("dynamic-only tariff should be present");
    assert_eq!(
        Some(true),
        item.get("isDynamic").and_then(|v| v.as_bool()),
        "expected isDynamic == true for the dynamic-only tariff, got: {item:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v3_unchanged_by_dynamic_prices(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;

    let regular = TariffBuilder::new().standard(false).create(&pool).await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(regular.id)
        .create(&pool)
        .await;

    let dynamic_only = TariffBuilder::new().standard(false).create(&pool).await;
    DynamicChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(dynamic_only.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/tariffs")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let ids = identifiers(&json);

    assert!(
        ids.contains(&regular.pub_tariff_id.to_string()),
        "expected the regular tariff in v3, got: {json:?}"
    );
    assert!(
        !ids.contains(&dynamic_only.pub_tariff_id.to_string()),
        "v3 must not include dynamic-only MSP tariffs, got: {json:?}"
    );

    let item = tariffs_of(&json).first().expect("non-empty array");
    assert!(
        item.get("isDynamic").is_none(),
        "v3 must not expose the isDynamic field, got: {item:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_post_returns_200_and_json_object(pool: PgPool) {
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
        .post("/v4/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(
        json.is_object(),
        "expected response body to be a JSON object, got: {json:?}"
    );

    let first = tariffs_of(&json)
        .first()
        .unwrap_or_else(|| panic!("expected a non-empty tariff array, got: {json:?}"));

    for field in [
        "identifier",
        "name",
        "isStandard",
        "providerName",
        "lastUpdatedDate",
        "isCustomerOnly",
        "isAdHoc",
        "isDynamic",
    ] {
        assert!(
            first.get(field).is_some(),
            "expected `{field}` field in item, got: {first:?}"
        );
    }
}

#[sqlx::test]
async fn test_tariffs_v4_post_defaults_to_standard_true(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let standard_tariff = TariffBuilder::new().standard(true).create(&pool).await;
    let non_standard_tariff = TariffBuilder::new()
        .standard(false)
        .monthly_fee(9.99)
        .create(&pool)
        .await;

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
        .post("/v4/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    for tariff in tariffs_of(&json) {
        let is_standard = tariff
            .get("isStandard")
            .and_then(|v| v.as_bool())
            .expect("isStandard should be a boolean");
        assert!(
            is_standard,
            "expected all tariffs to be standard when standard defaults to true, got: {tariff:?}"
        );
    }

    let ids = identifiers(&json);
    assert!(
        !ids.contains(&non_standard_tariff.pub_tariff_id.to_string()),
        "expected the paid non-standard tariff to be excluded, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_post_with_standard_false_returns_all(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let standard_tariff = TariffBuilder::new().standard(true).create(&pool).await;
    let non_standard_tariff = TariffBuilder::new()
        .standard(false)
        .monthly_fee(9.99)
        .create(&pool)
        .await;

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
        .post("/v4/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let ids = identifiers(&json);

    assert!(
        ids.contains(&standard_tariff.pub_tariff_id.to_string()),
        "expected the standard tariff to be included, got: {json:?}"
    );
    assert!(
        ids.contains(&non_standard_tariff.pub_tariff_id.to_string()),
        "expected the non-standard tariff to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_post_supports_add_list(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let custom_tariff = TariffBuilder::new()
        .standard(false)
        .monthly_fee(9.99)
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
        .post("/v4/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let ids = identifiers(&json);

    assert!(
        ids.contains(&custom_tariff.pub_tariff_id.to_string()),
        "expected a non-standard tariff listed in `add` to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_post_supports_remove_list(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let standard_tariff = TariffBuilder::new().standard(true).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(standard_tariff.id)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "standard": true,
        "add": [],
        "remove": [standard_tariff.pub_tariff_id],
        "operatorIds": [operator.pub_network]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v4/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let ids = identifiers(&json);

    assert!(
        !ids.contains(&standard_tariff.pub_tariff_id.to_string()),
        "expected a tariff listed in `remove` to be excluded, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_post_supports_operator_ids_filter(pool: PgPool) {
    let operator1 = OperatorBuilder::new().create(&pool).await;
    let operator2 = OperatorBuilder::new().create(&pool).await;

    let tariff1 = TariffBuilder::new().standard(false).create(&pool).await;
    let tariff2 = TariffBuilder::new().standard(false).create(&pool).await;

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
        .post("/v4/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let ids = identifiers(&json);

    assert!(
        ids.contains(&tariff1.pub_tariff_id.to_string()),
        "expected the tariff of the requested operator to be included, got: {json:?}"
    );
    assert!(
        !ids.contains(&tariff2.pub_tariff_id.to_string()),
        "expected the tariff of a non-requested operator to be excluded, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_post_combines_add_and_remove(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new()
        .standard(false)
        .monthly_fee(9.99)
        .create(&pool)
        .await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "standard": true,
        "add": [tariff.pub_tariff_id],
        "remove": [tariff.pub_tariff_id],
        "operatorIds": [operator.pub_network]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v4/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let ids = identifiers(&json);

    assert!(
        !ids.contains(&tariff.pub_tariff_id.to_string()),
        "expected `remove` to win over `add` for the same tariff, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_post_returns_empty_array_when_no_tariffs(pool: PgPool) {
    let request_body = serde_json::json!({
        "standard": true,
        "add": [],
        "remove": [],
        "operatorIds": []
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v4/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert_eq!(
        0,
        tariffs_of(&json).len(),
        "expected an empty tariff array, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_post_defaults_operator_ids_to_empty(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let standard_tariff = TariffBuilder::new().standard(true).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(standard_tariff.id)
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
        .post("/v4/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let ids = identifiers(&json);

    assert!(
        ids.contains(&standard_tariff.pub_tariff_id.to_string()),
        "expected a standard tariff to be returned with empty operatorIds, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_post_includes_dynamic_only_msp(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let dynamic_only = TariffBuilder::new().standard(false).create(&pool).await;

    DynamicChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(dynamic_only.id)
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
        .post("/v4/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let ids = identifiers(&json);

    assert!(
        ids.contains(&dynamic_only.pub_tariff_id.to_string()),
        "expected a dynamic-only MSP tariff to match via the dynamic operator arm, got: {json:?}"
    );

    let item = find_tariff(&json, &dynamic_only.pub_tariff_id);
    assert_eq!(
        Some(true),
        item.get("isDynamic").and_then(|v| v.as_bool()),
        "expected isDynamic == true for the dynamic-only tariff, got: {item:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_post_is_dynamic_false_for_charge_price_only(pool: PgPool) {
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
        .post("/v4/tariffs", request_body)
        .await;

    let json: Value = result.json().await;
    let item = find_tariff(&json, &tariff.pub_tariff_id);

    assert_eq!(
        Some(false),
        item.get("isDynamic").and_then(|v| v.as_bool()),
        "expected isDynamic == false for a charge-price-only tariff, got: {item:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_post_is_dynamic_true_for_mixed_prices(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().standard(true).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .create(&pool)
        .await;
    DynamicChargePriceBuilder::new()
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
        .post("/v4/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    let item = find_tariff(&json, &tariff.pub_tariff_id);
    assert_eq!(
        Some(true),
        item.get("isDynamic").and_then(|v| v.as_bool()),
        "expected isDynamic == true for a tariff with both price kinds, got: {item:?}"
    );

    let occurrences = identifiers(&json)
        .iter()
        .filter(|id| *id == &tariff.pub_tariff_id.to_string())
        .count();
    assert_eq!(
        1, occurrences,
        "expected the tariff exactly once, got {occurrences} in: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_post_standard_false_includes_dynamic_only_msp(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let dynamic_only = TariffBuilder::new().standard(false).create(&pool).await;

    DynamicChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(dynamic_only.id)
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
        .post("/v4/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let ids = identifiers(&json);

    assert!(
        ids.contains(&dynamic_only.pub_tariff_id.to_string()),
        "expected the dynamic-inclusive price gate to admit a dynamic-only MSP, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_post_dynamic_operator_match_requires_location(pool: PgPool) {
    let requested_operator = OperatorBuilder::new().create(&pool).await;
    let other_operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().standard(false).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(other_operator.id)
        .tariff_id(tariff.id)
        .create(&pool)
        .await;

    sqlx::query(
        r#"
        INSERT INTO dynamic_charge_price (operator_id, tariff_id, c_type, price, updated)
        VALUES ($1, $2, $3, $4, now())
        "#,
    )
    .bind(requested_operator.id)
    .bind(tariff.id)
    .bind(ChargeType::AC)
    .bind(0.49)
    .execute(&pool)
    .await
    .expect("could not insert location-less dynamic_charge_price");

    let request_body = serde_json::json!({
        "standard": true,
        "add": [],
        "remove": [],
        "operatorIds": [requested_operator.pub_network]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v4/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let ids = identifiers(&json);

    assert!(
        !ids.contains(&tariff.pub_tariff_id.to_string()),
        "a dynamic price without any location must not match the operator, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v4_post_is_standard_deterministic_across_operators(pool: PgPool) {
    let operator1 = OperatorBuilder::new().create(&pool).await;
    let operator2 = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().standard(false).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator1.id)
        .tariff_id(tariff.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(operator2.id)
        .tariff_id(tariff.id)
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
        .post("/v4/tariffs", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let item = find_tariff(&json, &tariff.pub_tariff_id);

    assert_eq!(
        Some(true),
        item.get("isStandard").and_then(|v| v.as_bool()),
        "expected isStandard == true when any priced operator is requested, got: {item:?}"
    );
}

#[sqlx::test]
async fn test_tariffs_v3_post_unchanged_by_dynamic_prices(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;

    let regular = TariffBuilder::new().standard(false).create(&pool).await;
    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(regular.id)
        .create(&pool)
        .await;

    let dynamic_only = TariffBuilder::new().standard(false).create(&pool).await;
    DynamicChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(dynamic_only.id)
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
    let ids = identifiers(&json);

    assert!(
        ids.contains(&regular.pub_tariff_id.to_string()),
        "expected the regular tariff in v3, got: {json:?}"
    );
    assert!(
        !ids.contains(&dynamic_only.pub_tariff_id.to_string()),
        "v3 POST must not include dynamic-only MSP tariffs, got: {json:?}"
    );

    let item = tariffs_of(&json).first().expect("non-empty array");
    assert!(
        item.get("isDynamic").is_none(),
        "v3 POST must not expose the isDynamic field, got: {item:?}"
    );
}
