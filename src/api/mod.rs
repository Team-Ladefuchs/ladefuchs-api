use axum::{
    extract::{rejection::PathRejection, Path},
    Json,
};

use crate::db::charging::ChargeType;

pub mod charge_card;
pub mod cpo;
pub mod error;
pub mod handler;
pub mod middleware;
pub mod route;
pub mod util;

pub type ApiJson<T> = Result<Json<T>, error::ApiError>;
pub type ApiJsonList<T> = Result<Json<Vec<T>>, error::ApiError>;
pub type RequestCardPath = Result<Path<(String, ChargeType)>, PathRejection>;

pub enum CardVersion {
    V1,
    V2,
    V3,
}
