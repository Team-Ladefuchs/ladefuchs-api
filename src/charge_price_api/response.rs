use std::collections::HashMap;

use crate::db::{plug::Plug, tariff::Tariff};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MSPApiResult {
    pub id: uuid::Uuid,
    pub attributes: MspAttributes,
    pub relationships: HashMap<String, HashMap<String, TarifJson>>,
}

impl MSPApiResult {
    pub fn into_tariff(&self, msp_id: i32) -> Tariff {
        let relationship_id = self
            .relationships
            .get("tariff")
            .and_then(|tariffs| tariffs.get("data"))
            .unwrap()
            .id;

        Tariff {
            id: 0,
            relationship_id,
            slug_name: self.attributes.tariff_name.clone(),
            monthly_fee: self.attributes.total_monthly_fee,
            msp_id,
            url: self.attributes.url.as_ref().map(|u| u.to_string()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TarifJson {
    #[serde(rename = "type")]
    c_type: String,
    pub id: uuid::Uuid,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MspAttributes {
    pub provider: String,
    pub tariff_name: String,
    #[serde_as(as = "NoneAsEmptyString")]
    pub url: Option<url::Url>,
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
pub struct ApiResponse {
    pub cpo_id: i32,
    pub msps: Vec<MSPApiResult>,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct ResponseError {
    status: String,
    title: String,
}
