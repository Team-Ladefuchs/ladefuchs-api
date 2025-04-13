use sqlx::Connection;
use sqlx::PgConnection;

use crate::eco_movement::api::response::price::ConnectorPrice;

pub async fn save_multiple(
    connection: &mut PgConnection,
    connector_prices: Vec<ConnectorPrice>,
) -> Result<(), sqlx::Error> {
    let mut transaction = connection.begin().await?;
    for connector_price in connector_prices {
        save(&mut transaction, connector_price).await?;
    }
    transaction.commit().await?;
    Ok(())
}

async fn save(
    connection: &mut PgConnection,
    connector_price: ConnectorPrice,
) -> Result<(), sqlx::Error> {
    for price_id in connector_price.pricing_ids {
        sqlx::query_file!(
            "sql/insert/eco_movement/connector_price.sql",
            connector_price.location_id,
            price_id,
            connector_price.evse_uid,
            connector_price.connector_id
        )
        .execute(&mut *connection)
        .await?;
    }

    Ok(())
}

