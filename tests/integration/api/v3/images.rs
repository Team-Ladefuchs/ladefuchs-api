use axum::http::StatusCode;
use ladefuchs_api::fixtures::{
    banner::BannerBuilder, image::ImageBuilder, operator::OperatorBuilder, tariff::TariffBuilder,
};
use pretty_assertions::assert_eq;
use serde_json::Value;
use sqlx::PgPool;

use crate::helpers::TestClient;

#[sqlx::test]
async fn test_images_v3_returns_200_and_json_array(pool: PgPool) {
    let image = ImageBuilder::new().create(&pool).await;
    let _operator = OperatorBuilder::new()
        .image(Some(image.id))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/images")
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
        first.get("relationId").is_some(),
        "expected `relationId` field in item, got: {first:?}"
    );
    assert!(
        first.get("relationType").is_some(),
        "expected `relationType` field in item, got: {first:?}"
    );
    assert!(
        first.get("blake3sum").is_some(),
        "expected `blake3sum` field in item, got: {first:?}"
    );
    assert!(
        first.get("lastUpdatedDate").is_some(),
        "expected `lastUpdatedDate` field in item, got: {first:?}"
    );
    assert!(
        first.get("imageUrl").is_some(),
        "expected `imageUrl` field in item, got: {first:?}"
    );
}

#[sqlx::test]
async fn test_images_v3_returns_empty_array_when_no_images_exist(pool: PgPool) {
    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/images")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;

    assert!(json.is_array(), "expected array, got: {json:?}");
    assert_eq!(
        0,
        json.as_array().expect("array").len(),
        "expected empty list when no images exist, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_images_v3_includes_tariff_images(pool: PgPool) {
    let image = ImageBuilder::new().create(&pool).await;
    let tariff = TariffBuilder::new()
        .image(Some(image.id))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/images")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("array");

    let tariff_images: Vec<_> = arr
        .iter()
        .filter(|item| {
            item.get("relationType")
                .and_then(|v| v.as_str())
                .map(|s| s == "tariff")
                .unwrap_or(false)
        })
        .collect();

    assert!(
        !tariff_images.is_empty(),
        "expected at least one tariff image, got: {json:?}"
    );

    let found_tariff = tariff_images.iter().any(|item| {
        item.get("relationId")
            .and_then(|v| v.as_str())
            .map(|s| s == tariff.pub_tariff_id.to_string())
            .unwrap_or(false)
    });

    assert!(
        found_tariff,
        "expected tariff image to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_images_v3_includes_operator_images(pool: PgPool) {
    let image = ImageBuilder::new().create(&pool).await;
    let operator = OperatorBuilder::new()
        .image(Some(image.id))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/images")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("array");

    let operator_images: Vec<_> = arr
        .iter()
        .filter(|item| {
            item.get("relationType")
                .and_then(|v| v.as_str())
                .map(|s| s == "operator")
                .unwrap_or(false)
        })
        .collect();

    assert!(
        !operator_images.is_empty(),
        "expected at least one operator image, got: {json:?}"
    );

    let found_operator = operator_images.iter().any(|item| {
        item.get("relationId")
            .and_then(|v| v.as_str())
            .map(|s| s == operator.pub_network.to_string())
            .unwrap_or(false)
    });

    assert!(
        found_operator,
        "expected operator image to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_images_v3_includes_banner_images(pool: PgPool) {
    let image = ImageBuilder::new().create(&pool).await;
    let banner = BannerBuilder::new().image(image.id).create(&pool).await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/images")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("array");

    let banner_images: Vec<_> = arr
        .iter()
        .filter(|item| {
            item.get("relationType")
                .and_then(|v| v.as_str())
                .map(|s| s == "banner")
                .unwrap_or(false)
        })
        .collect();

    assert!(
        !banner_images.is_empty(),
        "expected at least one banner image, got: {json:?}"
    );

    let found_banner = banner_images.iter().any(|item| {
        item.get("relationId")
            .and_then(|v| v.as_str())
            .map(|s| s == banner.identifier.to_string())
            .unwrap_or(false)
    });

    assert!(
        found_banner,
        "expected banner image to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_images_v3_filters_out_soft_deleted_images(pool: PgPool) {
    let deleted_image = ImageBuilder::new().soft_delete(true).create(&pool).await;
    let _operator = OperatorBuilder::new()
        .image(Some(deleted_image.id))
        .create(&pool)
        .await;

    let active_image = ImageBuilder::new().soft_delete(false).create(&pool).await;
    let _active_operator = OperatorBuilder::new()
        .image(Some(active_image.id))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/images")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("array");

    let deleted_checksum = deleted_image.checksum.clone();
    let found_deleted = arr.iter().any(|item| {
        item.get("blake3sum")
            .and_then(|v| v.as_str())
            .map(|s| s == deleted_checksum)
            .unwrap_or(false)
    });

    assert!(
        !found_deleted,
        "expected soft-deleted image to not be included, got: {json:?}"
    );

    let active_checksum = active_image.checksum.clone();
    let found_active = arr.iter().any(|item| {
        item.get("blake3sum")
            .and_then(|v| v.as_str())
            .map(|s| s == active_checksum)
            .unwrap_or(false)
    });

    assert!(
        found_active,
        "expected active image to be included, got: {json:?}"
    );
}

#[sqlx::test]
async fn test_images_v3_uses_correct_image_url_format(pool: PgPool) {
    let image = ImageBuilder::new().create(&pool).await;
    let _operator = OperatorBuilder::new()
        .image(Some(image.id))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/images")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("array");

    assert!(
        !arr.is_empty(),
        "expected at least one image to verify URL format, got: {json:?}"
    );

    for item in arr {
        let image_url = item
            .get("imageUrl")
            .and_then(|v| v.as_str())
            .expect("imageUrl should be a string");
        let blake3sum = item
            .get("blake3sum")
            .and_then(|v| v.as_str())
            .expect("blake3sum should be a string");

        assert!(
            image_url.contains("/image/"),
            "expected image URL to use /image/ path, got: {image_url:?}"
        );
        assert!(
            image_url.ends_with(blake3sum),
            "expected image URL to end with blake3sum, got: {image_url:?}, blake3sum: {blake3sum:?}"
        );
    }
}

#[sqlx::test]
async fn test_images_v3_includes_all_relation_types(pool: PgPool) {
    let tariff_image = ImageBuilder::new().create(&pool).await;
    let _tariff = TariffBuilder::new()
        .image(Some(tariff_image.id))
        .create(&pool)
        .await;

    let operator_image = ImageBuilder::new().create(&pool).await;
    let _operator = OperatorBuilder::new()
        .image(Some(operator_image.id))
        .create(&pool)
        .await;

    let banner_image = ImageBuilder::new().create(&pool).await;
    let _banner = BannerBuilder::new()
        .image(banner_image.id)
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/images")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("array");

    let relation_types: std::collections::HashSet<_> = arr
        .iter()
        .filter_map(|item| item.get("relationType").and_then(|v| v.as_str()))
        .collect();

    assert!(
        relation_types.contains("tariff"),
        "expected tariff relation type, got: {relation_types:?}"
    );
    assert!(
        relation_types.contains("operator"),
        "expected operator relation type, got: {relation_types:?}"
    );
    assert!(
        relation_types.contains("banner"),
        "expected banner relation type, got: {relation_types:?}"
    );
}

#[sqlx::test]
async fn test_images_v3_serializes_last_updated_date_as_iso_8601(pool: PgPool) {
    let image = ImageBuilder::new().create(&pool).await;
    let _operator = OperatorBuilder::new()
        .image(Some(image.id))
        .create(&pool)
        .await;

    let result = TestClient::new(pool)
        .await
        .authorized()
        .get("/v3/images")
        .await;

    assert_eq!(StatusCode::OK, result.status());

    let json: Value = result.json().await;
    let arr = json.as_array().expect("array");

    assert!(
        !arr.is_empty(),
        "expected at least one image to verify date format, got: {json:?}"
    );

    for item in arr {
        let last_updated = item
            .get("lastUpdatedDate")
            .and_then(|v| v.as_str())
            .expect("lastUpdatedDate should be a string");

        assert!(
            !last_updated.is_empty(),
            "expected lastUpdatedDate to be a non-empty ISO 8601 string, got: {last_updated:?}"
        );

        assert!(
            last_updated.contains('T'),
            "expected ISO 8601 format with T separator, got: {last_updated:?}"
        );
    }
}
