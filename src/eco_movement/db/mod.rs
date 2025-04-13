use sqlx::PgConnection;

pub mod connector_prices;
pub mod location;
pub mod price;

#[derive(Debug, strum_macros::Display)]
pub enum Table {
    #[strum(to_string = "location")]
    Location,
    #[strum(to_string = "connector_price")]
    ConnectorPrice,
    #[strum(to_string = "price")]
    Price,
}
