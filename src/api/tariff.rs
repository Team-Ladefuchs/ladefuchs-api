use chrono::Utc;
use serde::{Deserialize, Serialize};

pub mod v3 {
    use super::*;
    use crate::api::serialize_iso_8601;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TariffResponse {
        pub tariffs: Vec<Tariff>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]

    pub struct Tariff {
        pub identifier: uuid::Uuid,
        pub name: String,
        #[serde(skip_serializing_if = "is_zero")]
        pub monthly_fee: f64,
        #[serde(skip_serializing_if = "String::is_empty")]
        pub note: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub image_url: Option<String>,
        pub is_standard: bool,
        pub provider_name: String,
        pub is_customer_only: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub affiliate_link_url: Option<String>,
        #[serde(serialize_with = "serialize_iso_8601")]
        pub last_updated_date: chrono::DateTime<Utc>,
    }

    fn is_zero(n: &f64) -> bool {
        n == &0.0
    }
}

// fn serialize_iso_8601<'de, S>(serializer: S) -> Result<String, D::Error>
// where S: Serializer {
//     let buf = String::deserialize(deserializer)?;

//     cron::Schedule::from_str(&buf).map_err(serde::de::Error::custom)
// }
