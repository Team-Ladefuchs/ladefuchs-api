use serde::Deserialize;
use sqlx::Postgres;

use super::charging::ChargeType;

#[derive(Debug, Clone, Deserialize)]
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
        // let a =   self.c_type.
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
