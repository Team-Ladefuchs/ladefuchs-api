use crate::inc_sql;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use strum_macros::IntoStaticStr;

use super::PGPoolConnection;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, sqlx::Type, IntoStaticStr)]
#[non_exhaustive]
pub enum VehicleType {
    #[serde(rename = "car")]
    Car,
    #[serde(skip)]
    Empty,
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

pub async fn get_vehicles(connection: &mut PGPoolConnection) -> Result<Vec<Vehicle>, sqlx::Error> {
    let vehicles = sqlx::query(inc_sql!("get/all_vehicles"))
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

#[cfg(test)]
mod _tests {

    use crate::{config, db::connect};

    use super::*;

    #[tokio::test]
    async fn test_get_cpo() {
        let config = config::read_config().unwrap();
        let pool = connect(&config.database_url).await.unwrap();
        let mut conn = pool.acquire().await.unwrap();
        let vehicles = get_vehicles(&mut conn).await;
        assert!(vehicles.is_ok());
        assert!(!vehicles.unwrap().is_empty());
    }
}
