use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::ladefuchs_db::plug::ChargeType;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ChargePrice {
    pub operator_id: i32,
    pub tariff_id: i32,
    pub c_type: ChargeType,
    pub price: f64,
    pub blocking_fee_start: i64,
    pub blocking_fee: f64,
    pub updated: DateTime<Utc>,
}

pub struct ChargePriceBuilder {
    operator_id: Option<i32>,
    tariff_id: Option<i32>,
    c_type: ChargeType,
    price: f64,
    blocking_fee_start: i64,
    blocking_fee: f64,
    updated: DateTime<Utc>,
}

impl Default for ChargePriceBuilder {
    fn default() -> Self {
        Self {
            operator_id: None,
            tariff_id: None,
            c_type: ChargeType::AC,
            price: 0.49,
            blocking_fee_start: 0,
            blocking_fee: 0.0,
            updated: Utc::now(),
        }
    }
}

impl ChargePriceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn operator_id(mut self, operator_id: i32) -> Self {
        self.operator_id = Some(operator_id);
        self
    }

    pub fn tariff_id(mut self, tariff_id: i32) -> Self {
        self.tariff_id = Some(tariff_id);
        self
    }

    pub fn c_type(mut self, c_type: ChargeType) -> Self {
        self.c_type = c_type;
        self
    }

    pub fn price(mut self, price: f64) -> Self {
        self.price = price;
        self
    }

    pub fn blocking_fee_start(mut self, blocking_fee_start: i64) -> Self {
        self.blocking_fee_start = blocking_fee_start;
        self
    }

    pub fn blocking_fee(mut self, blocking_fee: f64) -> Self {
        self.blocking_fee = blocking_fee;
        self
    }

    pub fn updated(mut self, updated: DateTime<Utc>) -> Self {
        self.updated = updated;
        self
    }

    pub async fn create(self, pool: &PgPool) -> ChargePrice {
        let operator_id = self
            .operator_id
            .expect("ChargePriceBuilder requires `operator_id`");
        let tariff_id = self
            .tariff_id
            .expect("ChargePriceBuilder requires `tariff_id`");

        sqlx::query_as(
            r#"
            INSERT INTO charge_price
                (operator_id, tariff_id, c_type, price, blocking_fee_start, blocking_fee, updated)
            VALUES
                ($1,         $2,        $3,    $4,    $5,                $6,           $7)
            ON CONFLICT (operator_id, tariff_id, c_type)
            DO UPDATE SET
                price = EXCLUDED.price,
                blocking_fee_start = EXCLUDED.blocking_fee_start,
                blocking_fee = EXCLUDED.blocking_fee,
                updated = EXCLUDED.updated
            RETURNING
                operator_id,
                tariff_id,
                c_type,
                price,
                blocking_fee_start,
                blocking_fee,
                updated
            "#,
        )
        .bind(operator_id)
        .bind(tariff_id)
        .bind(self.c_type)
        .bind(self.price)
        .bind(self.blocking_fee_start)
        .bind(self.blocking_fee)
        .bind(self.updated)
        .fetch_one(pool)
        .await
        .expect("could not insert charge_price fixture")
    }
}
