use axum::{
    extract::{rejection::PathRejection, Path},
    Json,
};

use crate::db::plug::ChargeType;

pub mod charge_conditions;
pub mod endpoint;
pub mod error;
pub mod img;
pub mod tariff;
pub type ApiJson<T> = Result<Json<T>, error::ApiError>;
pub type ApiJsonList<T> = Result<Json<Vec<T>>, error::ApiError>;
pub type RequestCardPath = Result<Path<(String, ChargeType)>, PathRejection>;
pub type RequestConditionPath = Result<Path<uuid::Uuid>, PathRejection>;

pub type AllCard<T> = Vec<ChargePriceMap<T>>;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargePriceMap<T> {
    pub operator: uuid::Uuid,
    pub ac: Vec<T>,
    pub dc: Vec<T>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionsFilterRequest {
    #[serde(alias = "cpos")]
    pub operator_ids: Vec<uuid::Uuid>,
    #[serde(default)]
    pub tariff_ids: Vec<uuid::Uuid>,
    #[serde(default = "default_charging_modes")]
    pub charging_modes: Vec<ChargeType>,
}

fn default_charging_modes() -> Vec<ChargeType> {
    vec![ChargeType::AC, ChargeType::DC]
}

pub fn json<T>(data: T) -> ApiJson<T> {
    Ok(axum::Json(data))
}

pub fn json_list<T>(data: Vec<T>) -> ApiJsonList<T> {
    json(data)
}
