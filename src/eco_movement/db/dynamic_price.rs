use crate::ladefuchs_db::{
    dynamic_price::{EcoDynamicPrice, EcoLocation},
    plug::ChargeType,
};
use sqlx::PgConnection;

pub async fn get_locations(connection: &mut PgConnection) -> Result<Vec<EcoLocation>, sqlx::Error> {
    let rows = sqlx::query_file_as!(EcoLocation, "sql/get/eco_movement/get_locations.sql")
        .fetch_all(&mut *connection)
        .await?;

    Ok(rows)
}

pub async fn get_dynamic_prices(
    connection: &mut PgConnection,
) -> Result<Vec<EcoDynamicPrice>, sqlx::Error> {
    let rows = sqlx::query_file_as!(
        EcoDynamicPrice,
        "sql/get/eco_movement/get_dynamic_prices.sql"
    )
    .fetch_all(&mut *connection)
    .await?;

    Ok(rows)
}
