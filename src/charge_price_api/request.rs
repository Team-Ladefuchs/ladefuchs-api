use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::db::{
    operator::{self},
    plug::Plug,
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

pub mod tariff {

    use charge_station::ChargePoint;
    use operator::admin::Operator;

    use super::*;
    type TariffsDetailJson = DataWrapper<Vec<TariffDetailAttribute>>;

    #[derive(Serialize, Debug, Clone)]
    pub struct TariffRelationship {
        pub tariffs: TariffsDetailJson,
    }

    #[derive(Serialize, Debug, Clone)]
    pub struct TariffDetailsRequest {
        pub attributes: TariffAttributes,
        #[serde(skip)]
        pub operator: Operator,
    }

    #[derive(Serialize, Debug, Clone)]
    pub struct FilterRequest {
        pub foreign_tariffs: bool,
        pub provider_customer_tariffs: bool,
        pub tariffs_without_prices: bool,
    }

    impl TariffDetailsRequest {
        pub fn new(operator: Operator, charge_point: ChargePoint) -> Self {
            Self {
                attributes: TariffAttributes {
                    station: TariffStation {
                        country: "DE",
                        operator: GenericAttribute {
                            id: operator.network,
                            r_type: "company",
                        },
                        charge_point,
                    },
                    filter: FilterRequest {
                        foreign_tariffs: false,
                        provider_customer_tariffs: true,
                        tariffs_without_prices: false,
                    },
                },
                operator,
            }
        }
    }

    #[derive(Serialize, Debug, Clone)]
    pub struct TariffAttributes {
        pub station: TariffStation,
        pub filter: FilterRequest,
    }

    #[derive(Serialize, Debug, Clone)]
    pub struct TariffStation {
        pub country: &'static str,
        pub operator: GenericAttribute,
        pub charge_point: ChargePoint,
    }

    #[allow(dead_code)]
    #[derive(Serialize, Debug, Clone)]
    pub struct TariffDetailAttribute {
        pub id: uuid::Uuid,
        #[serde(rename = "type")]
        pub r_type: &'static str,
    }
}

pub mod charge_station {
    use std::fmt;

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

    impl fmt::Display for ChargePoint {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "ChargePoint {{ power: {}, plug: {} }}",
                self.power, self.plug
            )
        }
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
