use super::{plug::ChargeType, vehicle::VehicleType, MyPool};

use serde::Serialize;

use ::chrono::serde::ts_seconds;
use chrono::Utc;

async fn get_by_vehicle_type(
    cpo_name: &str,
    charge_type: &ChargeType,
    vehicle_type: &VehicleType,
    pool: &MyPool,
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
    T: From<CardV3>,
{
    let cards = get_by_vehicle_type(cpo_name, charge_type, &VehicleType::Car, pool)
        .await?
        .into_iter()
        .map(T::from)
        .collect();
    Ok(cards)
}

pub async fn get_v1(
    charge_type: &ChargeType,
    cpo_name: &str,
    pool: &MyPool,
) -> Result<Vec<CardV1>, sqlx::Error> {
    let cards = get_by_vehicle_type(cpo_name, charge_type, &VehicleType::Empty, pool)
        .await?
        .into_iter()
        .map(CardV1::from)
        .collect();
    Ok(cards)
}

#[derive(Debug, Clone, Serialize)]
pub struct CardV3 {
    pub identifier: uuid::Uuid,
    pub provider: String,
    pub name: String,
    pub price: f64,
    pub monthly_fee: f64,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
    #[serde(skip)]
    pub legacy_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CardV2 {
    pub identifier: String,
    pub provider: String,
    pub name: String,
    pub price: f64,
    pub monthly_fee: f64,
    pub updated: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CardV1 {
    pub identifier: String,
    pub provider: String,
    pub name: String,
    pub price: f64,
    pub monthly_fee: f64,
}

impl From<CardV3> for CardV1 {
    fn from(card: CardV3) -> Self {
        Self {
            identifier: card.legacy_id,
            monthly_fee: card.monthly_fee,
            price: card.price,
            provider: card.provider,
            name: card.name,
        }
    }
}

impl From<CardV3> for CardV2 {
    fn from(card: CardV3) -> Self {
        Self {
            identifier: card.legacy_id,
            updated: card.updated.timestamp(),
            monthly_fee: card.monthly_fee,
            price: card.price,
            provider: card.provider,
            name: card.name,
        }
    }
}
