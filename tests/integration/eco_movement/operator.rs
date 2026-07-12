use ladefuchs_api::eco_movement::db;
use ladefuchs_api::fixtures::charge_price::ChargePriceBuilder;
use ladefuchs_api::fixtures::operator::OperatorBuilder;
use ladefuchs_api::fixtures::tariff::TariffBuilder;
use sqlx::PgPool;

#[sqlx::test]
async fn get_standard_with_no_prices_returns_only_standard_without_charge_price(pool: PgPool) {
    let with_price = OperatorBuilder::new()
        .standard(true)
        .slug_name("with-price")
        .create(&pool)
        .await;

    let without_price = OperatorBuilder::new()
        .standard(true)
        .slug_name("without-price")
        .create(&pool)
        .await;

    let non_standard = OperatorBuilder::new()
        .standard(false)
        .slug_name("non-standard")
        .create(&pool)
        .await;

    let tariff = TariffBuilder::new().create(&pool).await;

    ChargePriceBuilder::new()
        .operator_id(with_price.id)
        .tariff_id(tariff.id)
        .create(&pool)
        .await;

    ChargePriceBuilder::new()
        .operator_id(non_standard.id)
        .tariff_id(tariff.id)
        .create(&pool)
        .await;

    let mut conn = pool.acquire().await.unwrap();
    let names = db::operator::get_standard_with_no_prices(&mut conn)
        .await
        .expect("query should succeed");

    assert!(
        names.contains(&without_price.slug_name),
        "expected '{}' in {:?}",
        without_price.slug_name,
        names
    );

    assert!(
        !names.contains(&with_price.slug_name),
        "did not expect '{}' in {:?}",
        with_price.slug_name,
        names
    );

    assert!(
        !names.contains(&non_standard.slug_name),
        "did not expect non-standard operator '{}' in {:?}",
        non_standard.slug_name,
        names
    );
}
