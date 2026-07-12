use ladefuchs_api::eco_movement::{api::response::tariff::TariffType, db};
use ladefuchs_api::fixtures::eco_movement::tariff::EcoTariffStagingBuilder;
use sqlx::PgPool;

#[sqlx::test]
async fn tariff_save_persists_product_id(pool: PgPool) {
    let product_id = uuid::Uuid::new_v4();
    let mut conn = pool.acquire().await.unwrap();

    let api_tariff = ladefuchs_api::eco_movement::api::response::tariff::Tariff {
        id: Some(product_id),
        name: "Persisted Tariff".to_string(),
        description: "Test".to_string(),
        subscription_type: "FIXED".to_string(),
        subscription_fee_excl_vat: "0".to_string(),
        _type: TariffType::Msp,
        currency: "EUR".to_string(),
    };

    let staging_id = db::tariff::save(&mut conn, &api_tariff, "Test Provider", product_id)
        .await
        .expect("tariff::save should succeed");

    let stored_product_id: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT product_id FROM eco_movement.tariff WHERE id = $1")
            .bind(staging_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(stored_product_id, Some(product_id));
}

#[sqlx::test]
async fn tariff_save_updates_existing_when_product_id_matches(pool: PgPool) {
    let product_id = uuid::Uuid::new_v4();
    let mut conn = pool.acquire().await.unwrap();

    EcoTariffStagingBuilder::new()
        .product_id(Some(product_id))
        .name("Initial Name")
        .provider_name("Initial Provider")
        .create(&pool)
        .await;

    let api_tariff = ladefuchs_api::eco_movement::api::response::tariff::Tariff {
        id: Some(product_id),
        name: "Updated Name".to_string(),
        description: "Updated".to_string(),
        subscription_type: "FIXED".to_string(),
        subscription_fee_excl_vat: "0".to_string(),
        _type: TariffType::Msp,
        currency: "EUR".to_string(),
    };

    db::tariff::save(&mut conn, &api_tariff, "Updated Provider", product_id)
        .await
        .expect("tariff::save should succeed");

    let row: (String, String) =
        sqlx::query_as("SELECT name, provider_name FROM eco_movement.tariff WHERE product_id = $1")
            .bind(product_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(row.0, "Updated Name");
    assert_eq!(row.1, "Updated Provider");

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM eco_movement.tariff")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "should not create a duplicate row");
}
