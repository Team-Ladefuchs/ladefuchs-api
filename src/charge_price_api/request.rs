use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::db::{
    charge_price::ChargePrice,
    operator::{self},
    plug::{ChargeType, Plug},
};

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct DataWrapper<T>
where
    T: Debug,
{
    pub data: T,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct GenericAttribute {
    pub id: uuid::Uuid,
    #[serde(rename = "type")]
    pub r_type: &'static str,
}

pub mod condition {

    use charge_station::{ChargePoint, ChargeStationContext};

    use super::*;
    #[derive(Serialize, Debug, Clone)]
    pub struct PriceRequest {
        #[serde(rename = "type")]
        pub r_type: &'static str,
        pub attributes: PriceAttributes,
        pub relationships: PriceRelationship,
        #[serde(skip)]
        pub operator: operator::admin::Operator,
    }

    impl PriceRequest {
        pub fn new(operator: &operator::admin::Operator, relationships: PriceRelationship) -> Self {
            let mut charge_points = vec![];

            if operator.supported_types.contains(&ChargeType::AC) {
                charge_points.push(ChargePoint {
                    power: operator.power_ac,
                    plug: Plug::TYPE2,
                })
            }
            if operator.supported_types.contains(&ChargeType::DC) {
                charge_points.push(ChargePoint {
                    power: operator.power_dc,
                    plug: Plug::CCS,
                })
            }
            Self {
                operator: operator.clone(),
                r_type: "charge_price_request",
                attributes: PriceAttributes {
                    station: ChargeStationContext {
                        longitude: 0.0,
                        latitude: 0.0,
                        country: "DE",
                        network: operator.network,
                        charge_points: charge_points.clone(),
                    },
                    data_adapter: "chargeprice",
                    options: PriceOptions::default(),
                },
                relationships,
            }
        }
    }

    #[derive(Serialize, Debug, Clone)]
    pub struct PriceOptions {
        energy: u32,
        duration: u32,
        provider_customer_tariffs: bool,
        max_monthly_fees: f32,
    }

    impl Default for PriceOptions {
        fn default() -> Self {
            Self {
                energy: 1,
                duration: 1,
                provider_customer_tariffs: true,
                max_monthly_fees: 25.0,
            }
        }
    }
    #[derive(Serialize, Debug, Clone)]
    pub struct PriceAttributes {
        pub data_adapter: &'static str,
        pub station: charge_station::ChargeStationContext,
        pub options: PriceOptions,
    }

    #[derive(Serialize, Debug, Clone)]
    pub struct PriceRelationship {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub vehicle: Option<DataWrapper<GenericAttribute>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tariffs: Option<tariff::TariffsJson>,
    }

    impl PriceRelationship {
        pub fn new(vehicle_id: uuid::Uuid, tariff_id: uuid::Uuid) -> Self {
            Self {
                vehicle: Some(DataWrapper {
                    data: GenericAttribute {
                        id: vehicle_id,
                        r_type: "car",
                    },
                }),
                tariffs: Some(DataWrapper {
                    data: vec![GenericAttribute {
                        id: tariff_id,
                        r_type: "tariff",
                    }],
                }),
            }
        }
    }

    impl Default for PriceRelationship {
        fn default() -> Self {
            Self {
                vehicle: None,
                tariffs: None,
            }
        }
    }
}

pub mod tariff {
    use super::*;
    pub type TariffsJson = DataWrapper<Vec<GenericAttribute>>;
    type TariffsDetailJson = DataWrapper<Vec<TariffDetailAttribute>>;

    #[derive(Serialize, Debug, Clone)]
    pub struct TariffRelationship {
        pub tariffs: TariffsDetailJson,
    }

    #[derive(Serialize, Debug, Clone)]
    pub struct TariffDetailsRequest {
        pub attributes: TariffAttributes,
        pub relationships: TariffRelationship,
        #[serde(skip)]
        pub operator_network: uuid::Uuid,
    }

    impl TariffDetailsRequest {
        pub fn new(operator_network: uuid::Uuid, charge_prices: Vec<ChargePrice>) -> Self {
            Self {
                attributes: TariffAttributes {
                    station: TariffStation {
                        country: "DE",
                        operator: GenericAttribute {
                            id: operator_network,
                            r_type: "company",
                        },
                    },
                },
                operator_network,
                relationships: TariffRelationship {
                    tariffs: TariffsDetailJson {
                        data: charge_prices
                            .into_iter()
                            .map(|price| TariffDetailAttribute {
                                id: price.tariff_relation,
                                r_type: "tariff",
                                tariff_relation_id: price.tariff_relation,
                                tariff_id: price.tariff_id,
                            })
                            .collect::<Vec<_>>(),
                    },
                },
            }
        }
    }

    #[derive(Serialize, Debug, Clone)]
    pub struct TariffAttributes {
        pub station: TariffStation,
    }

    #[derive(Serialize, Debug, Clone)]
    pub struct TariffStation {
        pub country: &'static str,
        pub operator: GenericAttribute,
    }

    #[allow(dead_code)]
    #[derive(Serialize, Debug, Clone)]
    pub struct TariffDetailAttribute {
        pub id: uuid::Uuid,
        #[serde(rename = "type")]
        pub r_type: &'static str,
        #[serde(skip)]
        pub tariff_relation_id: uuid::Uuid,
        #[serde(skip)]
        pub tariff_id: i32,
    }
}

pub mod charge_station {
    use super::*;
    #[derive(Clone, Debug)]
    pub struct ChargeStationStatistic {
        pub ccs_count: i32,
        pub type2_count: i32,
    }
    #[derive(Serialize, Debug, Clone)]
    pub struct ChargeStationContext {
        pub longitude: f32,
        pub latitude: f32,
        pub country: &'static str,
        pub network: uuid::Uuid,
        pub charge_points: Vec<ChargePoint>,
    }
    #[derive(Serialize, Debug, Clone)]
    pub struct ChargePoint {
        pub power: i32,
        pub plug: Plug,
    }
}

pub mod feedback {

    use super::*;

    // ISO-639-1
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "lowercase")]
    #[non_exhaustive]
    pub enum LanguageCode {
        De,
    }

    impl Default for LanguageCode {
        fn default() -> Self {
            Self::De
        }
    }

    #[derive(Serialize, Debug)]
    #[serde(tag = "type", content = "attributes")]
    pub enum TypeAttribute {
        #[serde(rename = "wrong_price")]
        WrongPrice(WrongPriceAttribute),
        #[serde(rename = "other_feedback")]
        Other(OtherAttribute),
    }

    pub type FeedBackRequest = DataWrapper<TypeAttribute>;

    #[derive(Serialize, Debug)]
    pub struct WrongPriceAttribute {
        pub email: String,
        pub context: String,
        pub notes: String,
        pub language: LanguageCode,
        pub tariff: String,
        pub cpo: String,
        pub displayed_price: String, // (100): Price displayed in the app.
        pub actual_price: String, // (100): Either total price or price per kWh/minute. Whatever the user has at hand.
        pub poi_link: &'static str,
    }
    #[derive(Serialize, Debug)]
    pub struct OtherAttribute {
        pub email: String,
        pub context: String,
        pub notes: String,
        pub language: LanguageCode,
    }
}
