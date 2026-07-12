use sqlx::Connection;
use sqlx::PgConnection;

#[derive(Debug, strum_macros::Display)]
pub enum Table {
    #[strum(to_string = "location")]
    Location,
    #[strum(to_string = "connector_price")]
    ConnectorPrice,
    #[strum(to_string = "price")]
    Price,
    #[strum(to_string = "operator")]
    Operator,
    #[strum(to_string = "tariff")]
    Tariff,
}

mod connector;
pub mod connector_prices;
pub mod dynamic_price;
pub mod location;
pub mod operator;
pub mod price;
pub mod tariff;

pub async fn truncate(connection: &mut PgConnection, table: Table) -> Result<(), sqlx::Error> {
    let query = format!("TRUNCATE TABLE eco_movement.{} cascade", table);
    sqlx::query(&query).execute(connection).await?;

    Ok(())
}
