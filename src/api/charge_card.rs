use ::chrono::serde::ts_seconds;
use ::serde::Serialize;
use chrono::Utc;

#[derive(Debug, Clone, Serialize)]
pub struct V3 {
    pub identifier: uuid::Uuid,
    pub provider: String,
    pub name: String,
    pub price: f64,
    pub monthly_fee: f64,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct V2 {
    pub identifier: String,
    pub provider: String,
    pub name: String,
    pub price: f64,
    pub monthly_fee: f64,
    pub updated: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct V1 {
    pub identifier: String,
    pub provider: String,
    pub name: String,
    pub price: f64,
    pub monthly_fee: f64,
}

impl From<V3> for V1 {
    fn from(card: V3) -> Self {
        Self {
            identifier: card.provider.clone().to_lowercase(),
            monthly_fee: card.monthly_fee,
            price: card.price,
            provider: card.provider,
            name: card.name,
        }
    }
}
