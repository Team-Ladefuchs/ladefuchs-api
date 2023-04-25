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
pub type ApiJson<T> = Result<Json<T>, error::ApiError>;
pub type ApiJsonList<T> = Result<Json<Vec<T>>, error::ApiError>;
pub type RequestCardPath = Result<Path<(String, ChargeType)>, PathRejection>;
pub type CardV2List = Vec<ChargePriceMap>;

// Vec<CardV2>,
//     dc: Vec<CardV2>,

#[derive(Debug, serde::Deserialize)]
pub struct CardByCpo {
    pub cpos: Vec<uuid::Uuid>,
}

pub fn json<T>(data: T) -> ApiJson<T> {
    Ok(axum::Json(data))
}

pub fn json_list<T>(data: Vec<T>) -> ApiJsonList<T> {
    json(data)
}
