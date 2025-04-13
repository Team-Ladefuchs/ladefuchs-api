use sqlx::Connection;
use sqlx::PgConnection;

use crate::eco_movement::api::response::location::LocationData;

pub async fn save_multiple(
    connection: &mut PgConnection,
    locations: &[LocationData],
) -> Result<(), sqlx::Error> {
    let mut transaction = connection.begin().await?;
    for location in locations {
        save(&mut transaction, location).await?;
    }
    transaction.commit().await?;
    Ok(())
}

async fn save(connection: &mut PgConnection, location: &LocationData) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "sql/insert/eco_movement/location.sql",
        location.id,
        location.value
    )
    .execute(&mut *connection)
    .await?;
    Ok(())
}
