use self::error::ApiError;
use axum::Json;
use chrono::SecondsFormat;

pub mod affiliate;
pub mod announcement;
pub mod app_metrics;
pub mod banner;
pub mod charge_condition;
pub mod cp_legacy_ads;
pub mod error;
pub mod feedback;
pub mod image;
pub mod operator;
pub mod tariff;

pub type ApiJson<T> = Result<Json<T>, error::ApiError>;
pub type ApiJsonList<T> = Result<Json<Vec<T>>, error::ApiError>;

pub fn json<T>(data: T) -> ApiJson<T> {
    Ok(axum::Json(data))
}

pub fn json_list<T>(data: Vec<T>) -> ApiJsonList<T> {
    json(data)
}

pub fn serialize_iso_8601<S>(
    value: &chrono::DateTime<chrono::Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_rfc3339_opts(SecondsFormat::Secs, true))
}

pub fn serialize_option_iso_8601<S>(
    value: &Option<chrono::DateTime<chrono::Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(v) => serializer.serialize_str(&v.to_rfc3339_opts(SecondsFormat::Secs, true)),
        None => serializer.serialize_none(),
    }
}

pub async fn handler_404() -> ApiError {
    ApiError::NotFound
}
