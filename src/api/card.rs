use ::chrono::serde::ts_seconds;
use chrono::Utc;
use serde::Serialize;

use crate::db::{plug::ChargeType, tariff::v1::Provider};

pub mod v3 {
    use super::*;
    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Card {
        pub blocking_fee_start: i64,
        pub blocking_fee: f64,
        pub identifier: uuid::Uuid,
        pub image: Option<String>,
        pub monthly_fee: f64,
        pub provider: Provider,
        pub note: String,
        #[serde(skip)]
        pub legacy_id: String,
        #[serde(skip)]
        pub c_type: ChargeType,
        pub price: f64,
        #[serde(rename = "name")]
        pub tariff_name: String,
        #[serde(rename = "url")]
        pub tariff_url: Option<String>,
        #[serde(with = "ts_seconds")]
        pub updated: chrono::DateTime<Utc>,
    }
}

pub mod v2 {
    use super::*;

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Card {
        pub blocking_fee_start: i64,
        pub blocking_fee: f64,
        pub identifier: uuid::Uuid,
        pub image: Option<String>,
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

    impl From<v3::Card> for Card {
        fn from(value: v3::Card) -> Self {
            Self {
                blocking_fee_start: value.blocking_fee_start,
                blocking_fee: value.blocking_fee,
                identifier: value.identifier,
                image: value.image,
                legacy_id: value.provider.name.to_string(),
                tariff_name: value.tariff_name,
                msp: value.provider.identifier,
                monthly_fee: value.monthly_fee,
                provider: value.provider.name,
                note: value.note,
                price: value.price,
                tariff_url: value.tariff_url,
                updated: value.updated,
            }
        }
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

    impl From<v3::Card> for Card {
        fn from(card: v3::Card) -> Self {
            Self {
                identifier: normalize_name(&card.legacy_id),
                price: card.price,
                provider: card.provider.name,
                name: card.tariff_name,
                updated: card.updated.timestamp(),
            }
        }
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
