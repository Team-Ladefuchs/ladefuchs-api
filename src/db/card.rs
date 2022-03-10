use super::{charging::ChargeType, MyPool};
use ::chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{types::chrono, Postgres};

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
            "sql/insert_charge_price.sql",
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
#[derive(Debug, Clone, Serialize)]
pub struct ChargeCardv3 {
    identifier: uuid::Uuid,
    provider: String,
    name: String,
    price: f64,
    monthly_fee: f64,
    updated: chrono::DateTime<Utc>,
}

async fn get(
    charge_type: &ChargeType,
    cpo_name: &str,
    pool: &MyPool,
) -> Result<Vec<ChargeCardv3>, sqlx::Error> {
    tracing::log::debug!("{:?} {}", charge_type = charge_type, cpo_name = cpo_name);
    let charge_type: &'static str = charge_type.into();

    let cards = sqlx::query_file_as!(ChargeCardv3, "sql/get_prices.sql", charge_type, cpo_name)
        .fetch_all(pool)
        .await?;
    tracing::log::debug!("{}", cards_len = cards.len());
    Ok(cards)
}

pub async fn get_v3(
    charge_type: &ChargeType,
    cpo_name: &str,
    pool: &MyPool,
) -> Result<Vec<ChargeCardv3>, sqlx::Error> {
    get(charge_type, cpo_name, pool).await
}

#[derive(Debug, Clone, Serialize)]
pub struct ChargeCardv2 {
    identifier: String,
    provider: String,
    name: String,
    price: f64,
    monthly_fee: f64,
    updated: i64,
}

impl From<ChargeCardv3> for ChargeCardv2 {
    fn from(card: ChargeCardv3) -> Self {
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

pub async fn get_v2(
    charge_type: &ChargeType,
    cpo_name: &str,
    pool: &MyPool,
) -> Result<Vec<ChargeCardv2>, sqlx::Error> {
    get_with(charge_type, cpo_name, pool).await
}

pub async fn get_with<T>(
    charge_type: &ChargeType,
    cpo_name: &str,
    pool: &MyPool,
) -> Result<Vec<T>, sqlx::Error>
where
    T: From<ChargeCardv3>,
{
    let cards = get(charge_type, cpo_name, pool)
        .await?
        .into_iter()
        .map(T::from)
        .collect();
    Ok(cards)
}

#[derive(Debug, Clone, Serialize)]
pub struct ChargeCardv1 {
    identifier: String,
    provider: String,
    name: String,
    price: f64,
    monthly_fee: f64,
}

pub async fn get_v1(
    charge_type: &ChargeType,
    cpo_name: &str,
    pool: &MyPool,
) -> Result<Vec<ChargeCardv1>, sqlx::Error> {
    get_with(charge_type, cpo_name, pool).await
}

impl From<ChargeCardv3> for ChargeCardv1 {
    fn from(card: ChargeCardv3) -> Self {
        Self {
            identifier: card.provider.clone().to_lowercase(),
            monthly_fee: card.monthly_fee,
            price: card.price,
            provider: card.provider,
            name: card.name,
        }
    }
}
