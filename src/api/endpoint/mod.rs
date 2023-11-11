use serde::Deserialize;

use super::error::ApiError;

pub mod affiliate;
pub mod charge_conditions;
pub mod images;
pub mod operators;
pub mod tariffs;

pub async fn handler_404() -> ApiError {
    ApiError::NotFound
}

#[derive(Deserialize, Debug)]
pub struct QueryFilter {
    #[serde(default)]
    pub standard: bool,
}
