use axum::{
    extract::{rejection::PathRejection, Path},
    Json,
};

use crate::db::charge_price::ChargePriceMap;
use crate::db::plug::ChargeType;

pub mod card;
pub mod endpoint;
pub mod error;
pub mod img;
pub mod tariff;
pub type ApiJson<T> = Result<Json<T>, error::ApiError>;
pub type ApiJsonList<T> = Result<Json<Vec<T>>, error::ApiError>;
pub type RequestCardPath = Result<Path<(String, ChargeType)>, PathRejection>;
pub type AllCard<T> = Vec<ChargePriceMap<T>>;

#[derive(Debug, serde::Deserialize)]
pub struct CardByOperatorsAndTariffs {
    #[serde(alias = "cpos")]
    pub operators: Vec<uuid::Uuid>,
    #[serde(default)]
    pub tariffs: Vec<uuid::Uuid>,
}

pub fn json<T>(data: T) -> ApiJson<T> {
    Ok(axum::Json(data))
}

pub fn json_list<T>(data: Vec<T>) -> ApiJsonList<T> {
    json(data)
}
