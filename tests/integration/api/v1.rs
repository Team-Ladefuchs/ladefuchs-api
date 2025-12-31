use axum::http::StatusCode;
use ladefuchs_api::fixtures::{
    charge_price::ChargePriceBuilder, operator::OperatorBuilder, tariff::TariffBuilder,
};
use pretty_assertions::assert_eq;
use serde_json::Value;
use sqlx::PgPool;

use crate::helpers::TestClient;

#[sqlx::test]
async fn test_cards_v1_returns_200_and_json_array_for_existing_operator(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;

    let tariff = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.42)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get(format!("/cards/de/{}/AC", operator.name))
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

    let first = arr.first().expect("non-empty array");
    assert!(
        first.get("identifier").is_some(),
        "expected `identifier` field in item, got: {first:?}"
    );
    assert!(
        first.get("name").is_some(),
        "expected `name` field in item, got: {first:?}"
    );
    assert!(
        first.get("provider").is_some(),
        "expected `provider` field in item, got: {first:?}"
    );
    assert!(
        first.get("price").is_some(),
        "expected `price` field in item, got: {first:?}"
    );
    assert!(
        first.get("updated").is_some(),
        "expected `updated` field in item, got: {first:?}"
    );
}

#[sqlx::test]
async fn test_cards_v1_operator_not_found_returns_404(pool: PgPool) {
    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/cards/de/this-operator-should-not-exist/AC")
        .await;

    assert_eq!(StatusCode::NOT_FOUND, result.status());
}

#[sqlx::test]
async fn test_cards_v1_invalid_charge_type_returns_400(pool: PgPool) {
    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/cards/de/anything/INVALID")
        .await;

    assert_eq!(StatusCode::BAD_REQUEST, result.status());
}

#[sqlx::test]
async fn test_cards_v1_missing_path_params_returns_404(pool: PgPool) {
    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/cards/de")
        .await;

    assert_eq!(StatusCode::NOT_FOUND, result.status());
}

#[sqlx::test]
async fn test_cards_v1_filters_out_non_standard_operator(pool: PgPool) {
    let operator = OperatorBuilder::new().standard(false).create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.55)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get(format!("/cards/de/{}/AC", operator.name))
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(json.is_array(), "expected array, got: {json:?}");
    assert_eq!(
        0,
        json.as_array().expect("array").len(),
        "expected empty list for non-standard operator, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_cards_v1_filters_out_hidden_tariff(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().hide(true).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.33)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get(format!("/cards/de/{}/AC", operator.name))
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(json.is_array(), "expected array, got: {json:?}");
    assert_eq!(
        0,
        json.as_array().expect("array").len(),
        "expected empty list for hidden tariff, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_operators_v1_all_returns_200_and_includes_enabled_and_disabled(pool: PgPool) {
    let enabled = OperatorBuilder::new().standard(true).create(&pool).await;
    let disabled = OperatorBuilder::new().standard(false).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(enabled.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(disabled.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/operators/all")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("expected JSON array");

    let names = arr
        .iter()
        .filter_map(|v| v.get("name").and_then(|name| name.as_str()))
        .collect::<Vec<_>>();

    assert!(
        names.contains(&enabled.name.to_lowercase().as_str()),
        "expected enabled operator to be included, got: {json:?}"
    );
    assert!(
        names.contains(&disabled.name.to_lowercase().as_str()),
        "expected disabled operator to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_operators_v1_enabled_returns_200_and_only_enabled(pool: PgPool) {
    let enabled = OperatorBuilder::new().standard(true).create(&pool).await;
    let disabled = OperatorBuilder::new().standard(false).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(enabled.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(disabled.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/operators/enabled")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("expected JSON array");

    let names = arr
        .iter()
        .filter_map(|v| v.get("name").and_then(|name| name.as_str()))
        .collect::<Vec<_>>();

    assert!(
        names.contains(&enabled.name.to_lowercase().as_str()),
        "expected enabled operator to be included, got: {json:?}"
    );

    assert!(
        !names.contains(&disabled.name.to_lowercase().as_str()),
        "expected disabled operator to not be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_operators_v1_disabled_returns_200_and_only_disabled(pool: PgPool) {
    let disabled = OperatorBuilder::new().standard(false).create(&pool).await;
    let enabled = OperatorBuilder::new().standard(true).create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(disabled.id)
        .create(&pool)
        .await;
    ChargePriceBuilder::new()
        .operator_id(enabled.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/operators/disabled")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("expected JSON array");

    let names = arr
        .iter()
        .filter_map(|v| v.get("name").and_then(|name| name.as_str()))
        .collect::<Vec<_>>();

    assert!(
        !names.contains(&enabled.name.to_lowercase().as_str()),
        "expected enabled operator to not be included, got: {json:?}"
    );

    assert!(
        names.contains(&disabled.name.to_lowercase().as_str()),
        "expected disabled operator to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_operators_v1_invalid_filter_returns_400(pool: PgPool) {
    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/operators/invalid-filter")
        .await;

    assert_eq!(StatusCode::BAD_REQUEST, result.status());
}
