use std::fmt::Debug;

use serde::Serialize;
use uuid::timestamp::context;

use crate::db::{
    cpo,
    plug::{ChargeType, Plug},
    tariff::TariffsWithBlockingFee,
};

#[derive(Serialize, Debug, Clone)]
pub struct DataWrapper<T>
where
    T: Serialize + Debug,
{
    pub data: T,
}

#[derive(Serialize, Debug, Clone)]
pub struct PriceRequest {
    #[serde(rename = "type")]
    pub r_type: &'static str,
    pub attributes: PriceAttributes,
    pub relationships: PriceRelationship,
    #[serde(skip)]
    pub cpo_name: String,
    #[serde(skip)]
    pub cpo_id: i32,
}

impl PriceRequest {
    pub fn new(cpo: &cpo::CPO, relationships: PriceRelationship) -> Self {
        let mut charge_points = vec![];

        if cpo.supported_types.contains(&ChargeType::AC) {
            charge_points.push(ChargePoint {
                power: cpo.power_ac,
                plug: Plug::TYPE2,
            })
        }
        if cpo.supported_types.contains(&ChargeType::DC) {
            charge_points.push(ChargePoint {
                power: cpo.power_ac,
                plug: Plug::CCS,
            })
        }
        Self {
            cpo_id: cpo.id,
            cpo_name: cpo.slug_name.clone(),
            r_type: "charge_price_request",
            attributes: PriceAttributes {
                station: PriceStation {
                    longitude: 0.0,
                    latitude: 0.0,
                    country: "DE",
                    network: cpo.network,
                    charge_points: charge_points.clone(),
                },
                data_adapter: "chargeprice",
                options: Options::default(),
            },
            relationships,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            energy: 1,
            duration: 1,
            max_monthly_fees: 0.0,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct PriceRelationship {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vehicle: Option<DataWrapper<GenericAttribute>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tariffs: Option<TariffsJson>,
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

type TariffsJson = DataWrapper<Vec<GenericAttribute>>;

#[derive(Serialize, Debug, Clone)]
pub struct PriceAttributes {
    pub data_adapter: &'static str,
    pub station: PriceStation,
    pub options: Options,
}

#[derive(Serialize, Debug, Clone)]
pub struct PriceStation {
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

#[derive(Serialize, Debug, Clone)]
pub struct Options {
    energy: u32,
    duration: u32,
    max_monthly_fees: f32,
}

#[derive(Serialize, Debug, Clone)]
pub struct VehicleJson {
    pub data: VehicleData,
}

#[derive(Serialize, Debug, Clone)]
pub struct VehicleData {
    pub id: uuid::Uuid,
    #[serde(rename = "type")]
    pub c_type: &'static str,
}

#[derive(Serialize, Debug, Clone)]
pub struct TariffDetailsRequest {
    pub attributes: TariffAttributes,
    pub relationships: TariffRelationship,
    #[serde(skip)]
    pub context: TariffsWithBlockingFee,
}

impl TariffDetailsRequest {
    pub fn new(value: TariffsWithBlockingFee) -> Self {
        Self {
            attributes: TariffAttributes {
                station: TariffStation {
                    country: "DE",
                    operator: GenericAttribute {
                        id: value.cpo_network,
                        r_type: "company",
                    },
                },
            },
            relationships: TariffRelationship {
                tariffs: TariffsJson {
                    data: vec![GenericAttribute {
                        id: value.relationship_id,
                        r_type: "tariff",
                    }],
                },
            },
            context: value,
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

#[derive(Serialize, Debug, Clone)]
pub struct GenericAttribute {
    id: uuid::Uuid,
    #[serde(rename = "type")]
    r_type: &'static str,
}

#[derive(Serialize, Debug, Clone)]
pub struct TariffRelationship {
    tariffs: TariffsJson,
}
