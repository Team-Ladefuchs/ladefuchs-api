use axum::http::StatusCode;
use chrono::Utc;
use ladefuchs_api::fixtures::{
    banner::BannerBuilder, charge_price::ChargePriceBuilder, operator::OperatorBuilder,
    tariff::TariffBuilder,
};
use pretty_assertions::assert_eq;
use serde_json::Value;
use sqlx::PgPool;

use crate::helpers::TestClient;

#[sqlx::test]
async fn test_operators_v2_all_returns_200_and_includes_enabled_and_disabled(pool: PgPool) {
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
        .get("/v2/operators/all")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("expected JSON array");

    let identifiers = arr
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        identifiers.contains(&enabled.pub_network.to_string().as_str()),
        "expected enabled operator to be included, got: {json:?}"
    );
    assert!(
        identifiers.contains(&disabled.pub_network.to_string().as_str()),
        "expected disabled operator to be included, got: {json:?}"
    );

    let first = arr.first().expect("non-empty array");
    assert!(
        first.get("identifier").is_some(),
        "expected `identifier` field in item, got: {first:?}"
    );
    assert!(
        first.get("displayName").is_some(),
        "expected `displayName` field in item, got: {first:?}"
    );
    assert!(
        first.get("updated").is_some(),
        "expected `updated` field in item, got: {first:?}"
    );
}

#[sqlx::test]
async fn test_operators_v2_enabled_returns_200_and_only_enabled(pool: PgPool) {
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
        .get("/v2/operators/enabled")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("expected JSON array");

    let identifiers = arr
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        identifiers.contains(&enabled.pub_network.to_string().as_str()),
        "expected enabled operator to be included, got: {json:?}"
    );

    assert!(
        !identifiers.contains(&disabled.pub_network.to_string().as_str()),
        "expected disabled operator to not be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_operators_v2_disabled_returns_200_and_only_disabled(pool: PgPool) {
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
        .get("/v2/operators/disabled")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("expected JSON array");

    let identifiers = arr
        .iter()
        .filter_map(|v| v.get("identifier").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        !identifiers.contains(&enabled.pub_network.to_string().as_str()),
        "expected enabled operator to not be included, got: {json:?}"
    );

    assert!(
        identifiers.contains(&disabled.pub_network.to_string().as_str()),
        "expected disabled operator to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_operators_v2_invalid_filter_returns_400(pool: PgPool) {
    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v2/operators/invalid-filter")
        .await;

    assert_eq!(StatusCode::BAD_REQUEST, result.status());
}

#[sqlx::test]
async fn test_cards_v2_returns_200_and_json_array_for_existing_operator(pool: PgPool) {
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
        .get(format!("/v2/cards/de/{}/AC", operator.name))
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
    assert!(
        first.get("blockingFee").is_some(),
        "expected `blockingFee` field in item, got: {first:?}"
    );
    assert!(
        first.get("blockingFeeStart").is_some(),
        "expected `blockingFeeStart` field in item, got: {first:?}"
    );
    assert!(
        first.get("monthlyFee").is_some(),
        "expected `monthlyFee` field in item, got: {first:?}"
    );
    assert!(
        first.get("msp").is_some(),
        "expected `msp` field in item, got: {first:?}"
    );
    assert!(
        first.get("note").is_some(),
        "expected `note` field in item, got: {first:?}"
    );
}

#[sqlx::test]
async fn test_cards_v2_operator_not_found_returns_404(pool: PgPool) {
    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v2/cards/de/this-operator-should-not-exist/AC")
        .await;

    assert_eq!(StatusCode::NOT_FOUND, result.status());
}

#[sqlx::test]
async fn test_cards_v2_invalid_charge_type_returns_400(pool: PgPool) {
    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v2/cards/de/anything/INVALID")
        .await;

    assert_eq!(StatusCode::BAD_REQUEST, result.status());
}

#[sqlx::test]
async fn test_cards_v2_missing_path_params_returns_404(pool: PgPool) {
    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v2/cards/de")
        .await;

    assert_eq!(StatusCode::NOT_FOUND, result.status());
}

#[sqlx::test]
async fn test_cards_v2_filters_out_non_standard_operator(pool: PgPool) {
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
        .get(format!("/v2/cards/de/{}/AC", operator.name))
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
async fn test_cards_v2_filters_out_hidden_tariff(pool: PgPool) {
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
        .get(format!("/v2/cards/de/{}/AC", operator.name))
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
async fn test_banners_returns_200_and_json_array(pool: PgPool) {
    let banner = BannerBuilder::new().create(&pool).await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/banners")
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
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        ids.contains(&banner.identifier.to_string().as_str()),
        "expected banner to be included in response, got: {json:?}"
    );

    let first = arr.first().expect("non-empty array");
    assert!(
        first.get("id").is_some(),
        "expected `id` field in item, got: {first:?}"
    );
    assert!(
        first.get("link").is_some(),
        "expected `link` field in item, got: {first:?}"
    );
    assert!(
        first.get("image").is_some(),
        "expected `image` field in item, got: {first:?}"
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
        first.get("updated").is_some(),
        "expected `updated` field in item, got: {first:?}"
    );
    assert!(
        first.get("filename").is_some(),
        "expected `filename` field in item, got: {first:?}"
    );
}

#[sqlx::test]
async fn test_banners_returns_empty_array_when_no_banners_exist(pool: PgPool) {
    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/banners")
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
async fn test_banners_filters_out_expired_banners(pool: PgPool) {
    let now = Utc::now();
    let expired_banner = BannerBuilder::new()
        .expiration(now - chrono::Duration::days(1))
        .starts(now - chrono::Duration::days(10))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/banners")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(json.is_array(), "expected array, got: {json:?}");
    let arr = json.as_array().expect("array");

    let ids = arr
        .iter()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        !ids.contains(&expired_banner.identifier.to_string().as_str()),
        "expected expired banner to not be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_banners_filters_out_future_banners(pool: PgPool) {
    let now = Utc::now();
    let future_banner = BannerBuilder::new()
        .starts(now + chrono::Duration::days(1))
        .expiration(now + chrono::Duration::days(10))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/banners")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(json.is_array(), "expected array, got: {json:?}");
    let arr = json.as_array().expect("array");

    let ids = arr
        .iter()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        !ids.contains(&future_banner.identifier.to_string().as_str()),
        "expected future banner to not be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_banners_includes_active_banners(pool: PgPool) {
    let now = Utc::now();
    let active_banner = BannerBuilder::new()
        .starts(now - chrono::Duration::days(1))
        .expiration(now + chrono::Duration::days(10))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/banners")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(json.is_array(), "expected array, got: {json:?}");
    let arr = json.as_array().expect("array");

    let ids = arr
        .iter()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
        .collect::<Vec<_>>();

    assert!(
        ids.contains(&active_banner.identifier.to_string().as_str()),
        "expected active banner to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_cards_v2_post_returns_200_and_json_array(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.42)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "operatorIds": [operator.pub_network]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v2/cards/de", request_body)
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
        first.get("operator").is_some(),
        "expected `operator` field in item, got: {first:?}"
    );
    assert!(
        first.get("ac").is_some(),
        "expected `ac` field in item, got: {first:?}"
    );
    assert!(
        first.get("dc").is_some(),
        "expected `dc` field in item, got: {first:?}"
    );

    let operator_id = first
        .get("operator")
        .and_then(|v| v.as_str())
        .expect("operator field should be a string");
    assert_eq!(
        operator.pub_network.to_string(),
        operator_id,
        "expected operator ID to match, got: {first:?}"
    );

    let ac_array = first
        .get("ac")
        .and_then(|v| v.as_array())
        .expect("ac should be an array");
    if !ac_array.is_empty() {
        let ac_card = ac_array.first().expect("non-empty ac array");
        assert!(
            ac_card.get("identifier").is_some(),
            "expected `identifier` field in AC card, got: {ac_card:?}"
        );
        assert!(
            ac_card.get("name").is_some(),
            "expected `name` field in AC card, got: {ac_card:?}"
        );
        assert!(
            ac_card.get("price").is_some(),
            "expected `price` field in AC card, got: {ac_card:?}"
        );
    }

    let dc_array = first
        .get("dc")
        .and_then(|v| v.as_array())
        .expect("dc should be an array");
    if !dc_array.is_empty() {
        let dc_card = dc_array.first().expect("non-empty dc array");
        assert!(
            dc_card.get("identifier").is_some(),
            "expected `identifier` field in DC card, got: {dc_card:?}"
        );
        assert!(
            dc_card.get("name").is_some(),
            "expected `name` field in DC card, got: {dc_card:?}"
        );
        assert!(
            dc_card.get("price").is_some(),
            "expected `price` field in DC card, got: {dc_card:?}"
        );
    }
}

#[sqlx::test]
async fn test_cards_v2_post_returns_empty_array_when_no_operators(pool: PgPool) {
    let request_body = serde_json::json!({
        "operatorIds": []
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v2/cards/de", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(json.is_array(), "expected array, got: {json:?}");
    assert_eq!(
        0,
        json.as_array().expect("array").len(),
        "expected empty list when no operators provided, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_cards_v2_post_returns_empty_arrays_for_nonexistent_operator(pool: PgPool) {
    let nonexistent_id = uuid::Uuid::new_v4();
    let request_body = serde_json::json!({
        "operatorIds": [nonexistent_id]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v2/cards/de", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(json.is_array(), "expected array, got: {json:?}");
    let arr = json.as_array().expect("array");

    if !arr.is_empty() {
        let first = arr.first().expect("non-empty array");
        let empty_vec: Vec<Value> = vec![];
        let ac = first
            .get("ac")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);
        let dc = first
            .get("dc")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);
        assert_eq!(
            0,
            ac.len(),
            "expected empty AC array for nonexistent operator, got: {json:?}"
        );
        assert_eq!(
            0,
            dc.len(),
            "expected empty DC array for nonexistent operator, got: {json:?}"
        );
    }
}

#[sqlx::test]
async fn test_cards_v2_post_supports_cpos_alias(pool: PgPool) {
    let operator = OperatorBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(operator.id)
        .tariff_id(tariff.id)
        .price(0.42)
        .create(&pool)
        .await;

    let request_body = serde_json::json!({
        "cpos": [operator.pub_network]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v2/cards/de", request_body)
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
        "expected non-empty array response when using cpos alias, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_cards_v2_post_handles_multiple_operators(pool: PgPool) {
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
        "operatorIds": [operator1.pub_network, operator2.pub_network]
    });

    let result = TestClient::new(pool)
        .await
        .authorized()
        .post("/v2/cards/de", request_body)
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(json.is_array(), "expected array, got: {json:?}");
    let arr = json.as_array().expect("array");
    assert!(
        !arr.is_empty(),
        "expected at least one operator in response, got: {json:?}"
    );

    let operator_ids: Vec<&str> = arr
        .iter()
        .filter_map(|v| v.get("operator").and_then(|id| id.as_str()))
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
