use crate::inc_sql;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, sqlx::Type)]
// #[serde(rename_all = "lowercase")]
pub enum VehicleType {
    #[serde(rename = "car")]
    Car,
}

impl Default for VehicleType {
    fn default() -> Self {
        VehicleType::Car
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct Vehicle {
    pub id: i32,
    pub uuid: uuid::Uuid,
    pub vehicle_type: VehicleType,
    pub name: String,
}

impl Default for Vehicle {
    fn default() -> Self {
        Self {
            id: 1,
            uuid: uuid::Uuid::parse_str("c1fd1277-5d77-416b-bb25-84bd21f57911").unwrap(),
            vehicle_type: Default::default(),
            name: "vehicle".into(),
        }
    }
}

pub async fn get_vehicles(
    connection: &mut sqlx::PgConnection,
) -> Result<Vec<Vehicle>, sqlx::Error> {
    let vehicles = sqlx::query(inc_sql!("get_all_vehicles"))
        .fetch_all(connection)
        .await?
        .into_iter()
        // todo log error or panic
        .filter_map(|row| Vehicle::from_row(&row).ok())
        .collect::<_>();
    Ok(vehicles)
}
