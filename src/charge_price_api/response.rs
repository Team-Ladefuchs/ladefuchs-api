use crate::db::{
    operator,
    plug::{ChargeType, Plug},
    tariff::ChargePriceTariff,
};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString, TimestampSeconds};

use super::request::DataWrapper;

pub mod condition {
    use super::*;
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct PriceResponse {
        pub id: uuid::Uuid,
        pub attributes: ProviderAttribute,
        pub relationships: tariff::TariffJson,
    }
    pub type PricesResponse = DataWrapper<Vec<PriceResponse>>;

    impl PriceResponse {
        pub fn relationship_id(&self) -> uuid::Uuid {
            self.relationships.tariff.data.id
        }

        pub fn into_tariff(
            &self,
            provider_name: String,
            filter_list: &[Regex],
            operator_is_standard: bool,
        ) -> ChargePriceTariff {
            let tariff_name = self.attributes.tariff_name.trim().to_string();
            let standard = {
                let attributes = &self.attributes;
                let all_filters_passed = filter_list.iter().all(|regex| {
                    let tariff_id = &self.relationships.tariff.data.id;
                    !regex.is_match(&attributes.tariff_name)
                        && !regex.is_match(&tariff_id.to_string())
                });
                let no_customer_tariff = !attributes.provider_customer_tariff;
                let zero_monthly_fee = attributes.total_monthly_fee == 0.0;

                let is_standard = if operator_is_standard {
                    all_filters_passed
                        && no_customer_tariff
                        && zero_monthly_fee
                        && operator_is_standard
                } else {
                    false
                };

                is_standard || self.attributes.direct_payment
            };

            ChargePriceTariff {
                id: 0,
                relationship_id: self.relationship_id(),
                slug_name: tariff_name,
                monthly_fee: self.attributes.total_monthly_fee,
                provider_name,
                provider_customer_only: self.attributes.provider_customer_tariff,
                url: self.attributes.url.as_ref().map(|u| u.to_string()),
                standard,
                ad_hoc: self.attributes.direct_payment,
                image: None,
            }
        }
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
}

pub mod company {

    use super::*;

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
}

pub mod tariff {

    use super::*;

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
        pub id: String,
        pub r#type: String,
        pub attributes: IncludedAttributes,
    }

    #[derive(Debug, Deserialize, Clone)]
    struct CompanyAttributes {
        name: String,
    }

    #[derive(Debug, Deserialize, Clone)]
    #[serde(untagged)]
    pub enum IncludedAttributes {
        Tariff(TariffAttributes),
        Company(CompanyAttributes),
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct TariffAttributes {
        pub name: String,
        pub total_monthly_fee: f64,
        pub is_direct_payment: bool,
        pub is_card_payment: bool,
        pub provider_customer_only: bool,
        pub currency: String,
        pub url: String,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct TariffDetailsSegments {
        pub charge_point_energy_type: Option<ChargeType>,
        pub price: f64,
        pub range_gte: Option<i64>,
        pub dimension: Dimension,
    }

    #[derive(Clone, Debug, Deserialize, PartialEq)]
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
