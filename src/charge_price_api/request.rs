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

    use crate::db::plug::ChargeType;

    use super::*;

    // ISO-639-1

    #[derive(Debug, strum_macros::Display, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum LanguageCode {
        #[strum(to_string = "de")]
        De,
    }

    impl Default for LanguageCode {
        fn default() -> Self {
            Self::De
        }
    }

    #[derive(Debug, sqlx::Type)]
    #[sqlx(type_name = "FeedbackKind")]
    #[sqlx(rename_all = "snake_case")]
    pub enum FeedbackKind {
        WrongPrice,
        Other,
    }

    #[derive(Debug)]
    pub struct Feedback {
        pub notes: String,
        pub language: LanguageCode,
        pub tariff_id: i32,
        pub operator_id: i32,
        pub kind: FeedbackKind,
        pub context: Option<WrongPriceContext>,
    }

    #[derive(Debug, Serialize)]
    pub struct WrongPriceContext {
        pub displayed_price: f32,
        pub actual_price: f32,
        pub charge_type: Option<ChargeType>,
    }
}
