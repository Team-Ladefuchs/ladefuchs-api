use super::error::ApiError;

pub mod affiliate;
pub mod cards;
pub mod images;
pub mod msps;
pub mod operators;

pub async fn handler_404() -> ApiError {
    ApiError::NotFound
}
