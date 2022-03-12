use axum::{Json, extract::{Path, rejection::PathRejection}};

use crate::db::charging::ChargeType;

pub mod charge_card;
pub mod cpo;
pub mod error;
pub mod handler;
pub mod middleware;
pub mod util;

pub type ApiJson<T> = Result<Json<T>, error::ApiError>;
pub type ApiJsonList<T> = Result<Json<Vec<T>>, error::ApiError>;
pub type RequestCardPath = Result<Path<(String, ChargeType)>, PathRejection>;
