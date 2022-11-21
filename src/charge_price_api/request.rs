use serde::Serialize;

use crate::db::{
    cpo,
    plug::{ChargeType, Plug},
};

#[derive(Serialize, Debug, Clone)]
pub struct RequestPayload {
    #[serde(rename = "type")]
    pub r_type: &'static str,
    pub attributes: Attributes,
    pub relationships: Relationship,
    #[serde(skip)]
    pub cpo_name: String,
    #[serde(skip)]
    pub cpo_id: i32,
}

impl RequestPayload {
    pub fn new(cpo: &cpo::CPO, relationships: Relationship) -> Self {
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
            attributes: Attributes {
                station: Station {
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
pub struct Relationship {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vehicle: Option<VehicleJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tariffs: Option<TariffsJson>,
}

impl Relationship {
    pub fn new(vehicle_id: uuid::Uuid, tariff_id: uuid::Uuid) -> Self {
        Self {
            vehicle: Some(VehicleJson {
                data: VehicleData {
                    id: vehicle_id,
                    c_type: "car",
                },
            }),
            tariffs: Some(TariffsJson {
                data: vec![Tariff {
                    id: tariff_id,
                    t_type: "tariff",
                }],
            }),
        }
    }
}

impl Default for Relationship {
    fn default() -> Self {
        Self {
            vehicle: None,
            tariffs: None,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct TariffsJson {
    pub data: Vec<Tariff>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Tariff {
    pub id: uuid::Uuid,
    #[serde(rename = "type")]
    pub t_type: &'static str,
}

#[derive(Serialize, Debug, Clone)]
pub struct Attributes {
    pub data_adapter: &'static str,
    pub station: Station,
    pub options: Options,
}

#[derive(Serialize, Debug, Clone)]
pub struct Station {
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
