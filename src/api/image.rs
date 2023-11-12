use chrono::{serde::ts_seconds, Utc};
use serde::Serialize;

use super::serialize_iso_8601;

pub mod v2 {
    use super::*;
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
    pub struct OperatorImage {
        pub cpo_identifier: uuid::Uuid,
        pub cpo_name: String,
        pub checksum: String,
        pub mime_type: String,
        #[serde(with = "ts_seconds")]
        pub updated: chrono::DateTime<Utc>,
        pub url: String,
    }
}

pub mod v3 {
    use super::*;
    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub enum RelationType {
        Tariff,
        Operator,
    }

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GenericImage {
        pub relation_id: uuid::Uuid,
        pub relation_type: RelationType,
        pub blake3sum: String,
        #[serde(serialize_with = "serialize_iso_8601")]
        pub last_updated_date: chrono::DateTime<Utc>,
        pub image_url: url::Url,
    }
}
