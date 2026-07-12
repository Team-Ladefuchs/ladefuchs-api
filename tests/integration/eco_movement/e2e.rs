use ladefuchs_api::eco_movement::importer::run_import;
use sqlx::PgPool;
use wiremock::MockServer;

use super::e2e_helpers::{build_state, mount_eco_endpoints};

const LOCATIONS: &str = include_str!("../../fixtures/eco_movement/locations.json");
const PRICES: &str = include_str!("../../fixtures/eco_movement/prices.json");
const CONNECTOR_PRICES: &str = include_str!("../../fixtures/eco_movement/connector_prices.json");

#[sqlx::test]
async fn run_import_imports_staging_and_rolls_back_when_too_few_prices(pool: PgPool) {
    let mock = MockServer::start().await;
    mount_eco_endpoints(&mock, LOCATIONS, PRICES, CONNECTOR_PRICES).await;

    let state = build_state(pool.clone(), &mock.uri());

    run_import(state)
        .await
        .expect("run_import should return Ok even on rollback path");

    let eco_locations: i64 = sqlx::query_scalar("SELECT count(*) FROM eco_movement.location")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        eco_locations, 1,
        "staging location should be persisted (separate transaction)"
    );

    let eco_prices: i64 = sqlx::query_scalar("SELECT count(*) FROM eco_movement.price")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(eco_prices, 1, "staging price should be persisted");

    let charge_prices: i64 = sqlx::query_scalar("SELECT count(*) FROM charge_price")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        charge_prices, 0,
        "public charge_price should remain empty after < 200 rollback"
    );

    let charging_locations: i64 = sqlx::query_scalar("SELECT count(*) FROM charging_location")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        charging_locations, 0,
        "public charging_location should remain empty after rollback"
    );
}
