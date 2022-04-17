use axum::{
    extract::{rejection::PathRejection, Path},
    Json,
};

use crate::db::plug::ChargeType;

pub mod card;
pub mod endpoint;
pub mod error;
pub mod middleware;
pub mod operator;
pub mod router;
pub mod util;
pub type ApiJson<T> = Result<Json<T>, error::ApiError>;
pub type ApiJsonList<T> = Result<Json<Vec<T>>, error::ApiError>;
pub type RequestCardPath = Result<Path<(String, ChargeType)>, PathRejection>;

pub enum CardVersion {
    V1,
    V2,
    V3,
}
