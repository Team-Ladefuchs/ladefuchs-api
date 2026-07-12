use ladefuchs_api::eco_movement::api::response::price::{
    ComponentType, Components, Elements, PriceData,
};
use ladefuchs_api::eco_movement::api::response::tariff::{Tariff, TariffType};
use ladefuchs_api::eco_movement::db;
use sqlx::PgPool;

fn build_price(currency: &str, product_id: Option<uuid::Uuid>, id: &str) -> PriceData {
    PriceData {
        id: id.to_string(),
        provider_name: "Test Provider".to_string(),
        tariff: Tariff {
            id: product_id,
            name: format!("Tariff for {}", id),
            description: String::new(),
            subscription_type: "FIXED".to_string(),
            subscription_fee_excl_vat: "0".to_string(),
            _type: TariffType::Msp,
            currency: currency.to_string(),
        },
        elements: vec![Elements {
            price_components: vec![Components {
                price_excl_vat: 0.5,
                vat: 19,
                step_size: 1,
                price_type: ComponentType::Energy,
            }],
            restrictions: None,
        }],
    }
}

#[sqlx::test]
async fn save_multiple_skips_prices_without_product_id(pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();

    let prices = vec![
        build_price("EUR", Some(uuid::Uuid::new_v4()), "with-product"),
        build_price("EUR", None, "without-product"),
    ];

    db::price::save_multiple(&mut conn, prices)
        .await
        .expect("save_multiple should succeed");

    let tariff_count: i64 = sqlx::query_scalar("SELECT count(*) FROM eco_movement.tariff")
        .fetch_one(&pool)
        .await
        .unwrap();
    let price_count: i64 = sqlx::query_scalar("SELECT count(*) FROM eco_movement.price")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(
        tariff_count, 1,
        "only the price with product_id should be saved"
    );

    assert_eq!(
        price_count, 1,
        "only the price with product_id should be saved"
    );
}

#[sqlx::test]
async fn save_multiple_writes_product_id_to_tariff(pool: PgPool) {
    let product_id = uuid::Uuid::new_v4();
    let mut conn = pool.acquire().await.unwrap();

    let prices = vec![build_price("EUR", Some(product_id), "p1")];

    db::price::save_multiple(&mut conn, prices)
        .await
        .expect("save_multiple should succeed");

    let stored: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT product_id FROM eco_movement.tariff LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(stored, Some(product_id));
}
