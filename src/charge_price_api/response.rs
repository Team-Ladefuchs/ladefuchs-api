use crate::db::{operator, plug::Plug, tariff::ChargePriceTariff};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString, TimestampSeconds};

use super::request::DataWrapper;

pub mod condition {

    use crate::db::charge_price::ChargePrice;

    use super::*;
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct PriceResponse {
        pub id: uuid::Uuid,
        pub attributes: ProviderAttribute,
        pub relationships: tariff::TariffJson,
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
        pub direct_payment: bool,
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
    pub struct ApiPriceResponse {
        pub operator: operator::admin::Operator,
        pub providers: Vec<PriceResponse>,
    }

    #[derive(Debug, Clone)]
    pub struct TariffPriceResponse {
        pub charge_prices: Vec<ChargePrice>,
        pub tariffs: Vec<tariff::TariffWithProvider>,
    }
}

pub mod company {

    use super::*;

    pub type CompanyResponse = DataWrapper<Vec<CompanyResult>>;

    #[derive(Clone, Debug, Deserialize)]
    pub struct CompanyResult {
        pub id: uuid::Uuid,
        pub attributes: CompanyAttribute,
    }

    impl CompanyResult {
        pub fn de_evs_ids(&self) -> Vec<String> {
            self.attributes
                .external_source_mapping
                .evse_operator_ids
                .clone()
                .map(|a| {
                    a.iter()
                        .filter(|d| d.starts_with("DE"))
                        .map(|d| d.to_owned())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        }
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
}

pub mod tariff {

    use std::sync::Arc;

    use super::*;
    use operator::admin::Operator;

    #[derive(Clone, Debug, Deserialize)]
    pub struct TariffDetailsResponses {
        pub data: Vec<TariffDetailsResponse>,
        pub included: Vec<TariffIncluded>,
    }

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
    pub struct TariffIncluded {
        pub id: uuid::Uuid,
        // pub r#type: String,
        pub attributes: IncludedAttributes,
        pub relationships: Option<EmpRelation>,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct EmpCompanyAttributes {
        pub name: String,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct EmpRelation {
        pub emp: DataWrapper<EmpData>,
        pub vehicle_brands: DataWrapper<Vec<serde_json::Value>>,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct EmpData {
        pub id: uuid::Uuid,
    }

    #[derive(Debug, Deserialize, Clone)]
    #[serde(untagged)]
    pub enum IncludedAttributes {
        Tariff(TariffAttributes),
        Company(EmpCompanyAttributes),
    }

    #[serde_as]
    #[derive(Clone, Debug, Deserialize)]
    pub struct TariffAttributes {
        pub name: String,
        pub total_monthly_fee: f64,
        pub is_direct_payment: bool,
        pub is_card_payment: bool,
        #[serde(default)]
        pub provider_customer_only: bool,
        #[serde_as(as = "NoneAsEmptyString")]
        pub url: Option<url::Url>,
    }

    // // aka EMP
    #[derive(Clone, Debug)]
    pub struct Provider {
        pub id: uuid::Uuid,
        pub name: String,
    }

    #[derive(Clone, Debug)]
    pub struct TariffWithProvider {
        pub id: uuid::Uuid,
        pub attributes: TariffAttributes,
        pub provider: Provider,
        pub operator: Arc<Operator>,
        pub is_brand_restricted: bool,
    }

    impl TariffWithProvider {
        pub fn into_tariff(&self) -> ChargePriceTariff {
            let tariff_name = self.attributes.name.trim().to_string();
            let standard = {
                let attributes = &self.attributes;

                let no_customer_tariff = !attributes.provider_customer_only;
                let zero_monthly_fee = attributes.total_monthly_fee == 0.0;
                let no_business = !tariff_name.to_lowercase().contains("business");

                no_customer_tariff
                    && self.operator.standard
                    && zero_monthly_fee
                    && !self.is_brand_restricted
                    && no_business
                    || self.attributes.is_direct_payment
            };

            ChargePriceTariff {
                id: 0,
                relationship_id: self.id,
                slug_name: tariff_name,
                monthly_fee: self.attributes.total_monthly_fee,
                provider_name: self.provider.name.clone(),
                provider_id: self.provider.id,
                provider_customer_only: self.attributes.provider_customer_only,
                url: self.attributes.url.as_ref().map(|u| u.to_string()),
                standard,
                ad_hoc: self.attributes.is_card_payment,
                brand_only: self.is_brand_restricted,
                image: None,
            }
        }
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct TariffDetailsSegments {
        pub price: f64,
        pub range_gte: Option<i64>,
        pub billing_increment: f64,
        pub time_of_day_start: Option<i64>,
        pub dimension: Dimension,
    }

    #[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
    pub enum Dimension {
        #[serde(alias = "kwh")]
        Kwh,
        #[serde(alias = "minute")]
        Minute,
        #[serde(alias = "session")]
        Session,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Relationship {
        #[serde(rename = "type")]
        c_type: String,
        pub id: uuid::Uuid,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct TariffJson {
        pub tariff: DataWrapper<condition::GenericResponse>,
    }
}

pub mod charge_station {
    use std::collections::HashMap;

    use crate::charge_price_api::request::charge_station::ChargeStationStatistic;

    use super::*;
    #[derive(Clone, Debug, Deserialize)]
    pub struct CompanyChargingStationData {
        pub attributes: ChargeStationDataResponse,
        pub relationships: serde_json::Value,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct ChargeStationDataResponse {
        pub plug: String,
        pub count: i32,
    }
    pub type ChargeStationResponse = DataWrapper<Vec<CompanyChargingStationData>>;
    pub type ChargingStationsStatists = HashMap<uuid::Uuid, ChargeStationStatistic>;
}

pub mod advertisement {
    use super::*;

    #[derive(Clone, Debug, Deserialize)]
    pub struct AdvertisementsResponse {
        pub banner_image_url: url::Url,
        pub cta_url: url::Url,
    }
}
