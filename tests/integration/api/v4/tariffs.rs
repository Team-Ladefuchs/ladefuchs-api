use axum::http::StatusCode;
use ladefuchs_api::fixtures::{
    charge_price::ChargePriceBuilder, dynamic_price::DynamicChargePriceBuilder,
    operator::OperatorBuilder, tariff::TariffBuilder,
};
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
