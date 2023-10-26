use crate::db::{
    operator::OperatorIntern,
    plug::{ChargeType, Plug},
    tariff::Tariff,
};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString, TimestampSeconds};

use super::request::DataWrapper;

pub type PricesResponse = DataWrapper<Vec<PriceResponse>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PriceResponse {
    pub id: uuid::Uuid,
    pub attributes: ProviderAttribute,
    pub relationships: TariffJson,
}

impl PriceResponse {
    pub fn into_tariff(
        &self,
        provider_name: String,
        operator: &OperatorIntern,
        filter_list: &[Regex],
    ) -> Tariff {
        let relationship_id = self.relationships.tariff.data.id;

        let standard = {
            let attributes = &self.attributes;
            let operator_enabled = operator.is_enabled;
            let all_filters_passed = filter_list.iter().all(|regex| {
                let tariff_id = &self.relationships.tariff.data.id;
                !regex.is_match(&attributes.tariff_name) && !regex.is_match(&tariff_id.to_string())
            });
            let no_customer_tariff = !attributes.provider_customer_tariff;
            let zero_monthly_fee = attributes.total_monthly_fee == 0.0;

            operator_enabled && all_filters_passed && no_customer_tariff && zero_monthly_fee
        };

        Tariff {
            id: 0,
            relationship_id,
            slug_name: self.attributes.tariff_name.clone(),
            monthly_fee: self.attributes.total_monthly_fee,
            provider_name,
            provider_customer_only: self.attributes.provider_customer_tariff,
            url: self.attributes.url.as_ref().map(|u| u.to_string()),
            standard,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Relationship {
    #[serde(rename = "type")]
    c_type: String,
    pub id: uuid::Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TariffJson {
    pub tariff: DataWrapper<GenericResponse>,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct GenericResponse {
    pub id: uuid::Uuid,
    #[serde(rename = "type")]
    pub r_type: String,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderAttribute {
    pub provider: String,
    pub tariff_name: String,
    #[serde_as(as = "NoneAsEmptyString")]
    pub url: Option<url::Url>,
    pub total_monthly_fee: f64,
    pub charge_point_prices: Vec<ChargePointPrice>,
    #[serde(default)]
    pub provider_customer_tariff: bool,
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
    pub operator: OperatorIntern,
    pub providers: Vec<PriceResponse>,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct ResponseError {
    status: String,
    title: String,
}

pub type CompanyResponse = DataWrapper<Vec<CompanyResult>>;

#[derive(Clone, Debug, Deserialize)]
pub struct CompanyResult {
    pub id: uuid::Uuid,
    pub attributes: CompanyAttribute,
}

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
pub struct CompanyAttribute {
    pub name: String,
    #[serde_as(as = "TimestampSeconds<i64>")]
    pub updated_at: DateTime<Utc>,
    pub is_cpo: bool,
    #[serde_as(as = "NoneAsEmptyString")]
    pub url: Option<String>,
    pub cpo_countries: Vec<String>,
    pub external_source_mapping: ExternalSource,
}

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
pub struct ExternalSource {
    pub evse_operator_ids: Option<Vec<String>>,
}

pub type TariffDetailsResponses = DataWrapper<Vec<TariffDetailsResponse>>;

#[derive(Clone, Debug, Deserialize)]
pub struct TariffDetailsAttribute {
    pub restricted_segments: Vec<TariffDetailsSegments>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TariffDetailsResponse {
    pub attributes: TariffDetailsAttribute,
    pub relationships: TariffJson,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TariffDetailsSegments {
    pub charge_point_energy_type: Option<ChargeType>,
    pub price: f64,
    pub dimension: DimenSion,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum DimenSion {
    #[serde(alias = "kwh")]
    Kwh,
    #[serde(alias = "minute")]
    Minute,
}

pub type ChargeStationResponse = DataWrapper<Vec<CompanyChargingStationData>>;

#[derive(Clone, Debug, Deserialize)]
pub struct CompanyChargingStationData {
    pub attributes: ChargeStationDataResponse,
    pub relationships: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChargeStationDataResponse {
    pub country: String,
    pub plug: String,
    pub count: i32,
}

#[derive(Clone, Debug)]
pub struct ChargeStation {
    pub operator_id: uuid::Uuid,
    pub ccs_count: i32,
    pub type2_count: i32,
}
