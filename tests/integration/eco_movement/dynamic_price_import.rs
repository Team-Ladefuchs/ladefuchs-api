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
    importer::dynamic_price::import(&mut tx, &None)
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
async fn import_keeps_one_row_per_price_when_locations_differ(pool: PgPool) {
    let eco_op = EcoOperatorBuilder::new().create(&pool).await;
    OperatorBuilder::new()
        .network(eco_op.id)
        .standard(true)
        .create(&pool)
        .await;

    let eco_tariff = EcoTariffStagingBuilder::new()
        .product_id(Some(uuid::Uuid::new_v4()))
        .create(&pool)
        .await;
    TariffBuilder::new()
        .relationship_id(eco_tariff.id)
        .create(&pool)
        .await;

    let connector = EcoConnectorBuilder::new().create(&pool).await;

    // Two locations of the same operator and tariff, without restrictions, so identical key except for the price.
    // Exactly the scenario that used to be combined into a single row
    let mut expected: Vec<(uuid::Uuid, f64)> = Vec::new();
    for price_excl_vat in [0.50_f64, 0.60_f64] {
        let eco_loc = EcoLocationBuilder::new(eco_op.id).create(&pool).await;
        let price = EcoPriceStagingBuilder::new(eco_tariff.id)
            .energy_only(price_excl_vat)
            .create(&pool)
            .await;

        EcoConnectorPriceBuilder::new(eco_loc.id, &price.id, &connector.evse_uid, &connector.id)
            .create(&pool)
            .await;

        expected.push((eco_loc.id, price_excl_vat * 1.19));
    }

    let mut conn = pool.acquire().await.unwrap();
    let mut tx = conn.begin().await.unwrap();
    importer::dynamic_price::import(&mut tx, &None)
        .await
        .expect("dynamic_price::import should succeed");
    tx.commit().await.unwrap();

    let price_count: i64 = sqlx::query_scalar("SELECT count(*) FROM dynamic_charge_price")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        price_count, 2,
        "both feed prices must survive as separate rows"
    );

    for (eco_location_id, expected_price) in expected {
        let prices: Vec<f64> = sqlx::query_scalar(
            "SELECT dp.price
             FROM charging_location AS cl
             INNER JOIN location_dynamic_price AS ldp ON ldp.location_id = cl.id
             INNER JOIN dynamic_charge_price AS dp ON dp.id = ldp.dynamic_price_id
             WHERE cl.eco_movement_id = $1",
        )
        .bind(eco_location_id)
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(
            prices.len(),
            1,
            "location {eco_location_id} should map to exactly one price row"
        );
        assert!(
            (prices[0] - expected_price).abs() < 1e-9,
            "location {eco_location_id} expected {expected_price}, got {}",
            prices[0]
        );
    }
}

#[sqlx::test]
async fn import_keeps_one_row_per_blocking_fee_when_locations_differ(pool: PgPool) {
    let eco_op = EcoOperatorBuilder::new().create(&pool).await;
    OperatorBuilder::new()
        .network(eco_op.id)
        .standard(true)
        .create(&pool)
        .await;

    let eco_tariff = EcoTariffStagingBuilder::new()
        .product_id(Some(uuid::Uuid::new_v4()))
        .create(&pool)
        .await;
    TariffBuilder::new()
        .relationship_id(eco_tariff.id)
        .create(&pool)
        .await;

    let connector = EcoConnectorBuilder::new().create(&pool).await;

    // Two locations with the SAME energy price but different blocking fees. Everything else in the key is identical,
    // so these used to combine into a single row and one of the two fees was served to both
    let mut expected: Vec<(uuid::Uuid, i64, f64)> = Vec::new();
    for (min_duration, parking_excl_vat) in [(60_i32, 0.10_f64), (120_i32, 0.20_f64)] {
        let eco_loc = EcoLocationBuilder::new(eco_op.id).create(&pool).await;
        let price = EcoPriceStagingBuilder::new(eco_tariff.id)
            .energy_with_parking(0.50, min_duration, parking_excl_vat)
            .create(&pool)
            .await;

        EcoConnectorPriceBuilder::new(eco_loc.id, &price.id, &connector.evse_uid, &connector.id)
            .create(&pool)
            .await;

        expected.push((eco_loc.id, i64::from(min_duration), parking_excl_vat * 1.19));
    }

    let mut conn = pool.acquire().await.unwrap();
    let mut tx = conn.begin().await.unwrap();
    importer::dynamic_price::import(&mut tx, &None)
        .await
        .expect("dynamic_price::import should succeed");
    tx.commit().await.unwrap();

    let price_count: i64 = sqlx::query_scalar("SELECT count(*) FROM dynamic_charge_price")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        price_count, 2,
        "both blocking fees must survive as separate rows"
    );

    for (eco_location_id, expected_start, expected_fee) in expected {
        let fees: Vec<(i64, f64)> = sqlx::query_as(
            "SELECT dp.blocking_fee_start, dp.blocking_fee
             FROM charging_location AS cl
             INNER JOIN location_dynamic_price AS ldp ON ldp.location_id = cl.id
             INNER JOIN dynamic_charge_price AS dp ON dp.id = ldp.dynamic_price_id
             WHERE cl.eco_movement_id = $1",
        )
        .bind(eco_location_id)
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(
            fees.len(),
            1,
            "location {eco_location_id} should map to exactly one price row"
        );
        assert_eq!(fees[0].0, expected_start);
        assert!(
            (fees[0].1 - expected_fee).abs() < 1e-9,
            "location {eco_location_id} expected {expected_fee}, got {}",
            fees[0].1
        );
    }
}

#[sqlx::test]
async fn import_replaces_row_when_blocking_fee_changes(pool: PgPool) {
    let eco_op = EcoOperatorBuilder::new().create(&pool).await;
    OperatorBuilder::new()
        .network(eco_op.id)
        .standard(true)
        .create(&pool)
        .await;

    let eco_tariff = EcoTariffStagingBuilder::new()
        .product_id(Some(uuid::Uuid::new_v4()))
        .create(&pool)
        .await;
    TariffBuilder::new()
        .relationship_id(eco_tariff.id)
        .create(&pool)
        .await;

    let eco_loc = EcoLocationBuilder::new(eco_op.id).create(&pool).await;
    let connector = EcoConnectorBuilder::new().create(&pool).await;
    let price = EcoPriceStagingBuilder::new(eco_tariff.id)
        .energy_with_parking(0.50, 60, 0.10)
        .create(&pool)
        .await;
    EcoConnectorPriceBuilder::new(eco_loc.id, &price.id, &connector.evse_uid, &connector.id)
        .create(&pool)
        .await;

    let run_import = async |pool: &PgPool| {
        let mut conn = pool.acquire().await.unwrap();
        let mut tx = conn.begin().await.unwrap();
        importer::dynamic_price::import(&mut tx, &None)
            .await
            .expect("dynamic_price::import should succeed");
        tx.commit().await.unwrap();
    };

    let stored = async |pool: &PgPool| -> Vec<(i64, f64)> {
        sqlx::query_as(
            "SELECT dp.blocking_fee_start, dp.blocking_fee
             FROM charging_location AS cl
             INNER JOIN location_dynamic_price AS ldp ON ldp.location_id = cl.id
             INNER JOIN dynamic_charge_price AS dp ON dp.id = ldp.dynamic_price_id
             WHERE cl.eco_movement_id = $1",
        )
        .bind(eco_loc.id)
        .fetch_all(pool)
        .await
        .unwrap()
    };

    run_import(&pool).await;
    assert_eq!(stored(&pool).await, vec![(60, 0.10 * 1.19)]);

    sqlx::query(
        r#"UPDATE eco_movement.price
           SET elements = '[{"price_components":[
                 {"price_excl_vat":0.50,"vat":19,"step_size":1,"price_type":"ENERGY"},
                 {"price_excl_vat":0.30,"vat":19,"step_size":1,"price_type":"PARKING_TIME"}],
               "restrictions":{"min_duration":90}}]'::json"#,
    )
    .execute(&pool)
    .await
    .unwrap();

    run_import(&pool).await;

    assert_eq!(
        stored(&pool).await,
        vec![(90, 0.30 * 1.19)],
        "location must follow the new blocking fee"
    );

    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM dynamic_charge_price")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(total, 1, "the superseded fee row must be swept, not linger");

    let mappings: i64 = sqlx::query_scalar("SELECT count(*) FROM location_dynamic_price")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(mappings, 1, "no orphaned junction rows");
}

#[sqlx::test]
async fn import_replaces_row_when_feed_price_changes(pool: PgPool) {
    let setup = seed_full_pipeline(&pool).await;

    let run_import = async |pool: &PgPool| {
        let mut conn = pool.acquire().await.unwrap();
        let mut tx = conn.begin().await.unwrap();
        importer::dynamic_price::import(&mut tx, &None)
            .await
            .expect("dynamic_price::import should succeed");
        tx.commit().await.unwrap();
    };

    let stored = async |pool: &PgPool| -> Vec<f64> {
        sqlx::query_scalar(
            "SELECT dp.price
             FROM charging_location AS cl
             INNER JOIN location_dynamic_price AS ldp ON ldp.location_id = cl.id
             INNER JOIN dynamic_charge_price AS dp ON dp.id = ldp.dynamic_price_id
             WHERE cl.eco_movement_id = $1",
        )
        .bind(setup.eco_location_id)
        .fetch_all(pool)
        .await
        .unwrap()
    };

    run_import(&pool).await;
    assert_eq!(stored(&pool).await, vec![0.50 * 1.19]);

    // The operator raises the price. Since the price is now part of the key, DO UPDATE no longer applies and a new
    // row is created. The old one must be removed by the sweep because this run did not touch its updated field
    sqlx::query(
        r#"UPDATE eco_movement.price
           SET elements = '[{"price_components":[{"price_excl_vat":0.60,"vat":19,"step_size":1,"price_type":"ENERGY"}],"restrictions":null}]'::json"#,
    )
    .execute(&pool)
    .await
    .unwrap();

    run_import(&pool).await;

    assert_eq!(
        stored(&pool).await,
        vec![0.60 * 1.19],
        "location must follow the new price"
    );

    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM dynamic_charge_price")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        total, 1,
        "the superseded price row must be swept, not linger"
    );

    let mappings: i64 = sqlx::query_scalar("SELECT count(*) FROM location_dynamic_price")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(mappings, 1, "no orphaned junction rows");
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
    importer::dynamic_price::import(&mut tx, &None)
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
    importer::dynamic_price::import(&mut tx, &None)
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

#[sqlx::test]
async fn empty_feed_skips_sweep_and_keeps_existing_rows(pool: PgPool) {
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

    let mut conn = pool.acquire().await.unwrap();
    let mut tx = conn.begin().await.unwrap();
    importer::dynamic_price::import(&mut tx, &None)
        .await
        .expect("dynamic_price::import should succeed");
    tx.commit().await.unwrap();

    let remaining: i64 =
        sqlx::query_scalar("SELECT count(*) FROM charging_location WHERE eco_movement_id = $1")
            .bind(stale_eco_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        remaining, 1,
        "existing row must survive an empty feed (sweep skipped)"
    );
}
