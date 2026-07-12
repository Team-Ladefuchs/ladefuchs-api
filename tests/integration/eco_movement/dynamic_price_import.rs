use ladefuchs_api::eco_movement::importer;
use ladefuchs_api::fixtures::eco_movement::{
    connector::EcoConnectorBuilder, connector_price::EcoConnectorPriceBuilder,
    location::EcoLocationBuilder, operator::EcoOperatorBuilder, price::EcoPriceStagingBuilder,
    tariff::EcoTariffStagingBuilder,
};
use ladefuchs_api::fixtures::{operator::OperatorBuilder, tariff::TariffBuilder};
use sqlx::{Acquire, PgPool};

struct Setup {
    public_operator_id: i32,
    public_tariff_id: i32,
    eco_location_id: uuid::Uuid,
    product_id: uuid::Uuid,
}

async fn seed_full_pipeline(pool: &PgPool) -> Setup {
    let eco_op = EcoOperatorBuilder::new().create(pool).await;
    let public_op = OperatorBuilder::new()
        .network(eco_op.id)
        .standard(true)
        .create(pool)
        .await;

    let product_id = uuid::Uuid::new_v4();
    let eco_tariff = EcoTariffStagingBuilder::new()
        .product_id(Some(product_id))
        .create(pool)
        .await;
    let public_tariff = TariffBuilder::new()
        .relationship_id(eco_tariff.id)
        .create(pool)
        .await;

    let eco_loc = EcoLocationBuilder::new(eco_op.id).create(pool).await;
    let connector = EcoConnectorBuilder::new().create(pool).await;

    let price = EcoPriceStagingBuilder::new(eco_tariff.id)
        .energy_only(0.50)
        .create(pool)
        .await;

    EcoConnectorPriceBuilder::new(eco_loc.id, &price.id, &connector.evse_uid, &connector.id)
        .create(pool)
        .await;

    Setup {
        public_operator_id: public_op.id,
        public_tariff_id: public_tariff.id,
        eco_location_id: eco_loc.id,
        product_id,
    }
}

#[sqlx::test]
async fn import_happy_path_populates_charging_location_and_dynamic_charge_price(pool: PgPool) {
    let setup = seed_full_pipeline(&pool).await;

    let mut conn = pool.acquire().await.unwrap();
    let mut tx = conn.begin().await.unwrap();
    importer::dynamic_price::import(&mut tx)
        .await
        .expect("dynamic_price::import should succeed");
    tx.commit().await.unwrap();

    let loc_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM charging_location WHERE eco_movement_id = $1")
            .bind(setup.eco_location_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(loc_count, 1, "expected one charging_location row");

    let price_row: (i32, i32, Option<uuid::Uuid>) = sqlx::query_as(
        "SELECT operator_id, tariff_id, product_id FROM dynamic_charge_price LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(price_row.0, setup.public_operator_id);
    assert_eq!(price_row.1, setup.public_tariff_id);
    assert_eq!(price_row.2, Some(setup.product_id));

    let mapping_count: i64 = sqlx::query_scalar("SELECT count(*) FROM location_dynamic_price")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(mapping_count, 1, "expected one junction row");
}

#[sqlx::test]
async fn sweep_stale_removes_pre_existing_untouched_rows(pool: PgPool) {
    let other_op = OperatorBuilder::new().create(&pool).await;
    let stale_eco_id = uuid::Uuid::new_v4();

    sqlx::query(
        "INSERT INTO charging_location (eco_movement_id, operator_id, geo, updated)
         VALUES ($1, $2, ST_SetSRID(ST_MakePoint(13.4, 52.5), 4326)::geography, now() - interval '1 hour')",
    )
    .bind(stale_eco_id)
    .bind(other_op.id)
    .execute(&pool)
    .await
    .unwrap();

    seed_full_pipeline(&pool).await;

    let mut conn = pool.acquire().await.unwrap();
    let mut tx = conn.begin().await.unwrap();
    importer::dynamic_price::import(&mut tx)
        .await
        .expect("dynamic_price::import should succeed");
    tx.commit().await.unwrap();

    let stale_remaining: i64 =
        sqlx::query_scalar("SELECT count(*) FROM charging_location WHERE eco_movement_id = $1")
            .bind(stale_eco_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        stale_remaining, 0,
        "stale charging_location should be swept"
    );
}

#[sqlx::test]
async fn sweep_stale_keeps_rows_touched_by_this_import(pool: PgPool) {
    let setup = seed_full_pipeline(&pool).await;

    sqlx::query(
        "INSERT INTO charging_location (eco_movement_id, operator_id, geo, updated)
         VALUES ($1, $2, ST_SetSRID(ST_MakePoint(13.4, 52.5), 4326)::geography, now() - interval '1 hour')",
    )
    .bind(setup.eco_location_id)
    .bind(setup.public_operator_id)
    .execute(&pool)
    .await
    .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let mut tx = conn.begin().await.unwrap();
    importer::dynamic_price::import(&mut tx)
        .await
        .expect("dynamic_price::import should succeed");
    tx.commit().await.unwrap();

    let remaining: i64 =
        sqlx::query_scalar("SELECT count(*) FROM charging_location WHERE eco_movement_id = $1")
            .bind(setup.eco_location_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(remaining, 1, "touched charging_location should be kept");
}
