use std::option::Option;

use serde::{Deserialize, Serialize};
use sqlx::Postgres;

use super::{charging::ChargeType, MyPool};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Price {
    pub cpo_id: i32,
    pub tarif_id: i32,
    pub c_type: ChargeType,
    pub price: f64,
    pub blocking_fee_start: i64,
}

impl Price {
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
pub struct ChargePrice {
    identifier: uuid::Uuid,
    provider: String,
    name: String,
    price: f64,
    monthly_fee: f64,
    updated: Option<std::string::String>,
}

pub async fn get(
    charge_type: ChargeType,
    cpo_name: &str,
    pool: &MyPool,
) -> Result<Vec<ChargePrice>, sqlx::Error> {
    tracing::log::debug!("{:?} {}", charge_type = charge_type, cpo_name = cpo_name);
    let charge_type: &'static str = charge_type.into();
    let a = sqlx::query_file_as!(ChargePrice, "sql/get_prices.sql", charge_type, cpo_name)
        .fetch_all(pool)
        .await?;
    Ok(a)
}
