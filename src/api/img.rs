use chrono::{serde::ts_seconds, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TariffImage {
    pub tariff_identifier: uuid::Uuid,
    pub tariff_name: String,
    pub checksum: String,
    pub mime_type: String,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpoImage {
    pub cpo_identifier: uuid::Uuid,
    pub cpo_name: String,
    pub checksum: String,
    pub mime_type: String,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
    pub url: String,
}
