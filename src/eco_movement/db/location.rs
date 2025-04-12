use sqlx::Connection;
use sqlx::PgConnection;

use crate::eco_movement::api::location::LocationData;

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
        "sql/insert/eco_movement/add_location.sql",
        location.id,
        location.value
    )
    .execute(&mut *connection)
    .await?;
    Ok(())
}

pub async fn truncate(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/insert/eco_movement/truncate_location.sql")
        .execute(&mut *connection)
        .await?;
    Ok(())
}
