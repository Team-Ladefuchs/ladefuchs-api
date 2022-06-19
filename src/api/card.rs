use ::chrono::serde::ts_seconds;
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CardV2 {
    pub identifier: uuid::Uuid,
    pub provider: String,
    pub name: String,
    pub price: f64,
    pub monthly_fee: f64,
    pub blocking_fee_start: i64,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
    #[serde(skip)]
    pub legacy_id: String,
    pub image: Option<String>,
    pub tarif_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CardV1 {
    pub identifier: String,
    pub provider: String,
    pub name: String,
    pub price: f64,
    pub updated: i64,
}

impl From<CardV2> for CardV1 {
    fn from(card: CardV2) -> Self {
        Self {
            identifier: card.legacy_id,
            price: card.price,
            provider: card.provider,
            name: card.name,
            updated: card.updated.timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Image {
    pub tariff_identifier: uuid::Uuid,
    pub checksum: String,
    pub mime_type: String,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
    pub url: String,
}
