use super::*;

use crate::eco_movement::api::response::location::{ConnectorType, Evse};

async fn save(connection: &mut PgConnection, evse: &Evse) -> Result<(), sqlx::Error> {
    for connector in evse
        .connectors
        .iter()
        .filter(|item| item.connector_type != ConnectorType::Other)
    {
        sqlx::query_file!(
            "sql/insert/eco_movement/connector.sql",
            connector.id,
            evse.uid,
            connector.power_type as _,
            connector.max_power,
            connector.connector_type as _
        )
        .execute(&mut *connection)
        .await?;
    }

    Ok(())
}

pub type ConnectorKey<'a> = (&'a str, &'a str);

pub async fn connector_exists<'a>(
    connection: &mut PgConnection,
    (connector_id, evse_id): ConnectorKey<'a>,
) -> Option<String> {
    sqlx::query_file_scalar!(
        "sql/insert/eco_movement/connector_by_id.sql",
        connector_id,
        evse_id
    )
    .fetch_optional(&mut *connection)
    .await
    .ok()
    .flatten()
}

pub async fn save_multiple(
    connection: &mut PgConnection,
    evses: &[Evse],
) -> Result<(), sqlx::Error> {
    let mut transaction = connection.begin().await?;

    for evse in evses {
        save(&mut transaction, evse).await?;
    }

    transaction.commit().await?;

    Ok(())
}
