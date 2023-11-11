use crate::db::plug::ChargeType;
use chrono::Utc;
use serde::Serialize;
pub mod v3 {
    use super::*;
    use crate::api::serialize_option_iso_8601;

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ChargeConditionResponse {
        #[serde(serialize_with = "serialize_option_iso_8601")]
        pub last_updated_date: Option<chrono::DateTime<chrono::Utc>>,
        pub charging_conditions: Vec<TariffConditions>,
    }

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TariffConditions {
        pub operator_id: uuid::Uuid,
        pub tariff_conditions: Vec<ChargeCondition>,
    }

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ChargeCondition {
        pub blocking_fee_start: i64,
        pub blocking_fee: f64,
        pub charging_mode: ChargeType,
        pub price_per_kwh: f64,
        pub tariff_id: uuid::Uuid,
        pub tariff_name: String,
        // #[serde(skip)]
        pub updated: chrono::DateTime<Utc>,
    }
}

pub mod v2 {
    use super::*;
    use chrono::serde::ts_seconds;

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Card {
        pub blocking_fee_start: i64,
        pub blocking_fee: f64,
        pub identifier: uuid::Uuid,
        pub image: Option<String>,
        #[serde(skip)]
        pub c_type: ChargeType,
        #[serde(skip)]
        pub legacy_id: String,
        #[serde(rename = "name")]
        pub tariff_name: String,
        pub msp: uuid::Uuid,
        pub monthly_fee: f64,
        pub provider: String,
        pub note: String,
        pub price: f64,
        #[serde(rename = "url")]
        pub tariff_url: Option<String>,
        #[serde(with = "ts_seconds")]
        pub updated: chrono::DateTime<Utc>,
    }
}

pub mod v1 {
    use super::*;

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Card {
        pub identifier: String,
        pub name: String,
        pub provider: String,
        pub price: f64,
        pub updated: i64,
    }

    impl From<v2::Card> for Card {
        fn from(card: v2::Card) -> Self {
            Self {
                identifier: normalize_name(&card.legacy_id),
                price: card.price,
                provider: card.provider,
                name: card.tariff_name,
                updated: card.updated.timestamp(),
            }
        }
    }
}

fn normalize_name(id: &str) -> String {
    let mut white_space_mode = false;
    id.trim()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .map(|c| c.to_ascii_lowercase())
        .filter_map(|c| {
            let ret = match c {
                c if c.is_whitespace() && !white_space_mode => {
                    white_space_mode = true;
                    Some('_')
                }
                c if c.is_whitespace() => None,
                'ä' => Some('a'),
                'ü' => Some('u'),
                'ö' => Some('o'),
                'ß' => Some('s'),
                _ => Some(c),
            };
            if !c.is_whitespace() {
                white_space_mode = false
            }
            ret
        })
        .collect()
}
