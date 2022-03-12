use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::db::{charging::Plug, tarif::Tarif};

pub type AllChargePrices = Vec<ApiResultWrapper>;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MSPApiResult {
    pub id: uuid::Uuid,
    pub attributes: MspAttributes,
    pub relationships: HashMap<String, HashMap<String, TarifJson>>,
}

impl MSPApiResult {
    pub fn into_tarif(&self, vehicle_id: i32, msp_id: i32) -> Tarif {
        let relationship_id = self
            .relationships
            .get("tariff")
            .and_then(|tarifs| tarifs.get("data"))
            .unwrap()
            .id;

        Tarif {
            relationship_id,
            vehicle_id,
            slug_name: self.attributes.tariff_name.clone(),
            monthly_fee: self.attributes.total_monthly_fee,
            msp_id,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TarifJson {
    #[serde(rename = "type")]
    c_type: String,
    id: uuid::Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MspAttributes {
    pub provider: String,
    pub tariff_name: String,
    pub total_monthly_fee: f64,
    pub charge_point_prices: Vec<ChargePointPrice>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChargePointPrice {
    pub power: f64,
    pub plug: Plug,
    pub price: f64,
    pub blocking_fee_start: Option<i64>,
    pub price_distribution: PriceDistribution,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PriceDistribution {
    pub kwh: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResultWrapper {
    pub vehicle_id: i32,
    pub cpo_id: i32,
    pub msps: Vec<MSPApiResult>,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct ResponseError {
    status: String,
    title: String,
}
