use sqlx::Connection;
use sqlx::PgConnection;

use crate::eco_movement::api::response::location::LocationData;
use crate::eco_movement::api::response::price::PriceData;

pub async fn save_multiple(
    connection: &mut PgConnection,
    prices: &[PriceData],
) -> Result<(), sqlx::Error> {
    let mut transaction = connection.begin().await?;
    for price in prices {
        save(&mut transaction, price).await?;
    }
    transaction.commit().await?;
    Ok(())
}

async fn save(connection: &mut PgConnection, price: &PriceData) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/insert/eco_movement/price.sql", price.id, price.value)
        .execute(&mut *connection)
        .await?;
    Ok(())
}
