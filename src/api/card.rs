use ::chrono::serde::ts_seconds;
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CardV3 {
    pub identifier: uuid::Uuid,
    pub provider: String,
    pub name: String,
    pub price: f64,
    pub monthly_fee: f64,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
    #[serde(skip)]
    pub legacy_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CardV2 {
    pub identifier: String,
    pub provider: String,
    pub name: String,
    pub price: f64,
    pub monthly_fee: f64,
    pub updated: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CardV1 {
    pub identifier: String,
    pub provider: String,
    pub name: String,
    pub price: f64,
    pub monthly_fee: f64,
}

impl From<CardV3> for CardV1 {
    fn from(card: CardV3) -> Self {
        Self {
            identifier: card.legacy_id,
            monthly_fee: card.monthly_fee,
            price: card.price,
            provider: card.provider,
            name: card.name,
        }
    }
}

impl From<CardV3> for CardV2 {
    fn from(card: CardV3) -> Self {
        Self {
            identifier: card.legacy_id,
            updated: card.updated.timestamp(),
            monthly_fee: card.monthly_fee,
            price: card.price,
            provider: card.provider,
            name: card.name,
        }
    }
}
