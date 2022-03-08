use std::collections::HashMap;

use serde::Serialize;

use crate::db::{charging::Plug, vehicle::VehicleType};

#[derive(Serialize, Debug, Clone)]
pub struct RequestPayload {
    #[serde(rename = "type")]
    pub r_type: &'static str,
    pub attributes: Attributes,
    pub relationships: HashMap<String, VehicleJson>,
    #[serde(skip)]
    pub cpo_name: String,
    #[serde(skip)]
    pub cpo_id: i32,
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
    pub power: u16,
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
    pub c_type: VehicleType,
}
