use ::chrono::serde::ts_seconds;
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CardV2 {
    pub blocking_fee_start: i64,
    pub identifier: uuid::Uuid,
    pub image: Option<String>,
    #[serde(skip)]
    pub legacy_id: String,
    #[serde(rename = "name")]
    pub tariff_name: String,
    pub monthly_fee: f64,
    pub provider: String,
    pub price: f64,
    #[serde(rename = "url")]
    pub tariff_url: Option<String>,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CardV1 {
    pub identifier: String,
    pub name: String,
    pub provider: String,
    pub price: f64,
    pub updated: i64,
}

impl From<CardV2> for CardV1 {
    fn from(card: CardV2) -> Self {
        Self {
            identifier: card.legacy_id,
            price: card.price,
            provider: card.provider,
            name: card.tariff_name,
            updated: card.updated.timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Image {
    pub tariff_identifier: uuid::Uuid,
    pub tariff_name: String,
    pub checksum: String,
    pub mime_type: String,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
    pub url: String,
}
