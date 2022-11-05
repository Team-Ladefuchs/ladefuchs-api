use crate::inc_sql;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgConnection};

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct Vehicle {
    pub id: uuid::Uuid,
    pub name: String,
    pub tariff_id: uuid::Uuid,
}

pub async fn get_vehicles(connection: &mut PgConnection) -> Result<Vec<Vehicle>, sqlx::Error> {
    let vehicles = sqlx::query(inc_sql!("get/vehicles"))
        .fetch_all(connection)
        .await?
        .into_iter()
        // todo log error or panic
        .filter_map(|row| match Vehicle::from_row(&row) {
            Ok(v) => Some(v),
            Err(err) => {
                tracing::error!(
                    info = "could not get vehicle",
                    reason = format_args!("{:#?}", err)
                );
                None
            }
        })
        .collect();
    Ok(vehicles)
}
