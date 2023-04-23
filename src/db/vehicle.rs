use serde::{Deserialize, Serialize};
use sqlx::PgConnection;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Vehicle {
    pub id: uuid::Uuid,
    pub name: String,
    pub tariff_id: uuid::Uuid,
}

pub async fn get_vehicles(connection: &mut PgConnection) -> Result<Vec<Vehicle>, sqlx::Error> {
    let vehicles = sqlx::query_file_as!(Vehicle, "sql/get/tariff/vehicles.sql")
        .fetch_all(connection)
        .await?;
    Ok(vehicles)
}
