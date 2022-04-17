use super::vehicle::VehicleType;
use crate::api::card::{self, CardV1, CardV3};
use crate::db::plug::ChargeType;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChargePrice {
    pub cpo_id: i32,
    pub tarif_id: i32,
    pub c_type: ChargeType,
    pub price: f64,
    pub blocking_fee_start: i64,
}

impl ChargePrice {
    pub async fn save(
        &self,
        transaction: &mut sqlx::Transaction<'_, Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        tracing::log::debug!("{:#?}", self);
        sqlx::query_file!(
            "sql/insert_update/charge_price.sql",
            self.cpo_id,
            self.tarif_id,
            self.c_type as ChargeType,
            self.price,
            self.blocking_fee_start
        )
        .execute(transaction)
        .await?;
        Ok(())
    }
}

async fn get_by_vehicle_type(
    connection: &mut PoolConnection<Postgres>,
    cpo_name: &str,
    charge_type: &ChargeType,
    vehicle_type: &VehicleType,
) -> Result<Vec<CardV3>, sqlx::Error> {
    let charge_type: &'static str = charge_type.into();
    let vehicle_type: &'static str = vehicle_type.into();
    let cards = sqlx::query_file_as!(
        CardV3,
        "sql/get/charge_price_by_vehicle_type.sql",
        cpo_name,
        vehicle_type,
        charge_type,
    )
    .fetch_all(connection)
    .await?;

    Ok(cards)
}

pub async fn get_with_ioniq<T>(
    connection: &mut PoolConnection<Postgres>,
    charge_type: &ChargeType,
    cpo_name: &str,
) -> Result<Vec<T>, sqlx::Error>
where
    T: From<card::CardV3>,
{
    let cards = get_by_vehicle_type(connection, cpo_name, charge_type, &VehicleType::Car)
        .await?
        .into_iter()
        .map(T::from)
        .collect();
    Ok(cards)
}

pub async fn get_v1(
    connection: &mut PoolConnection<Postgres>,
    charge_type: &ChargeType,
    cpo_name: &str,
) -> Result<Vec<CardV1>, sqlx::Error> {
    let cards = get_by_vehicle_type(connection, cpo_name, charge_type, &VehicleType::Empty)
        .await?
        .into_iter()
        .map(CardV1::from)
        .collect();
    Ok(cards)
}
