use crate::api::charge_card;

use super::{charging::ChargeType, vehicle::VehicleType, MyPool};

use serde::{Deserialize, Serialize};
use sqlx::Postgres;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Card {
    pub cpo_id: i32,
    pub tarif_id: i32,
    pub c_type: ChargeType,
    pub price: f64,
    pub blocking_fee_start: i64,
}

impl Card {
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

// useful later
// async fn get_by_vehicle_id(
//     charge_type: &ChargeType,
//     cpo_name: &str,
//     vehicle_pub_id: &uuid::Uuid,
//     pool: &MyPool,
// ) -> Result<Vec<ChargeCardv3>, sqlx::Error> {
//     let charge_type: &'static str = charge_type.into();

//     let cards = sqlx::query_file_as!(
//         ChargeCardv3,
//         "sql/get/charge_price_by_vehicle_id.sql",
//         cpo_name,
//         vehicle_pub_id,
//         charge_type,
//     )
//     .fetch_all(pool)
//     .await?;

//     Ok(cards)
// }

async fn get_by_vehicle_type(
    cpo_name: &str,
    charge_type: &ChargeType,
    vehicle_type: &VehicleType,
    pool: &MyPool,
) -> Result<Vec<charge_card::V3>, sqlx::Error> {
    let charge_type: &'static str = charge_type.into();
    let vehicle_type: &'static str = vehicle_type.into();
    let cards = sqlx::query_file_as!(
        charge_card::V3,
        "sql/get/charge_price_by_vehicle_type.sql",
        cpo_name,
        vehicle_type,
        charge_type,
    )
    .fetch_all(pool)
    .await?;

    Ok(cards)
}

pub async fn get_with_ioniq<T>(
    charge_type: &ChargeType,
    cpo_name: &str,
    pool: &MyPool,
) -> Result<Vec<T>, sqlx::Error>
where
    T: From<charge_card::V3>,
{
    let cards = get_by_vehicle_type(cpo_name, charge_type, &VehicleType::Car, pool)
        .await?
        .into_iter()
        .map(T::from)
        .collect();
    Ok(cards)
}

impl From<charge_card::V3> for charge_card::V2 {
    fn from(card: charge_card::V3) -> Self {
        Self {
            identifier: card.provider.clone().to_lowercase(),
            updated: card.updated.timestamp(),
            monthly_fee: card.monthly_fee,
            price: card.price,
            provider: card.provider,
            name: card.name,
        }
    }
}

pub async fn get_v1(
    charge_type: &ChargeType,
    cpo_name: &str,
    pool: &MyPool,
) -> Result<Vec<charge_card::V1>, sqlx::Error> {
    let cards = get_by_vehicle_type(cpo_name, charge_type, &VehicleType::Empty, pool)
        .await?
        .into_iter()
        .map(charge_card::V1::from)
        .collect();
    Ok(cards)
}
