use sqlx::PgPool;

use crate::{
    fixtures::{operator::OperatorBuilder, tariff::TariffBuilder},
    ladefuchs_db::plug::ChargeType,
};

#[derive(Debug, Clone)]
pub struct DynamicChargePrice {
    pub dynamic_price_id: i32,
    pub location_id: i64,
    pub operator_id: i32,
    pub tariff_id: i32,
}

pub struct DynamicChargePriceBuilder {
    operator_id: Option<i32>,
    tariff_id: Option<i32>,
    c_type: ChargeType,
    price: f64,
    latitude: f64,
    longitude: f64,
}

impl Default for DynamicChargePriceBuilder {
    fn default() -> Self {
        Self {
            operator_id: None,
            tariff_id: None,
            c_type: ChargeType::AC,
            price: 0.49,
            latitude: 52.5200,
            longitude: 13.4050,
        }
    }
}

impl DynamicChargePriceBuilder {
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

    pub fn coordinates(mut self, latitude: f64, longitude: f64) -> Self {
        self.latitude = latitude;
        self.longitude = longitude;
        self
    }

    pub async fn create(self, pool: &PgPool) -> DynamicChargePrice {
        let operator_id = if let Some(operator_id) = self.operator_id {
            operator_id
        } else {
            OperatorBuilder::new().create(pool).await.id
        };

        let tariff_id = if let Some(tariff_id) = self.tariff_id {
            tariff_id
        } else {
            TariffBuilder::new().create(pool).await.id
        };

        let (location_id,): (i64,) = sqlx::query_as(
            r#"
            INSERT INTO charging_location
                (eco_movement_id, operator_id, geo, address, city, postal_code, updated)
            VALUES
                ($1, $2, ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography, $5, $6, $7, now())
            RETURNING id
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(operator_id)
        .bind(self.longitude)
        .bind(self.latitude)
        .bind("Teststraße 1")
        .bind("Berlin")
        .bind("10115")
        .fetch_one(pool)
        .await
        .expect("could not insert charging_location fixture");

        let (dynamic_price_id,): (i32,) = sqlx::query_as(
            r#"
            INSERT INTO dynamic_charge_price
                (operator_id, tariff_id, c_type, price, updated)
            VALUES
                ($1, $2, $3, $4, now())
            RETURNING id
            "#,
        )
        .bind(operator_id)
        .bind(tariff_id)
        .bind(self.c_type)
        .bind(self.price)
        .fetch_one(pool)
        .await
        .expect("could not insert dynamic_charge_price fixture");

        sqlx::query(
            r#"
            INSERT INTO location_dynamic_price (location_id, dynamic_price_id, updated)
            VALUES ($1, $2, now())
            "#,
        )
        .bind(location_id)
        .bind(dynamic_price_id)
        .execute(pool)
        .await
        .expect("could not insert location_dynamic_price fixture");

        DynamicChargePrice {
            dynamic_price_id,
            location_id,
            operator_id,
            tariff_id,
        }
    }
}
