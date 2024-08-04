use crate::api::ApiJsonList;
use crate::api::{error::ApiError, serialize_option_iso_8601};
use crate::db::plug::ChargeType;
use crate::{api::json_list, db::charge_price, state::State};

use axum::extract::rejection::JsonRejection;
use axum::extract::rejection::PathRejection;
use axum::{extract::Path, Extension, Json};
use chrono::serde::ts_seconds;
use chrono::Utc;
use serde::Serialize;

use super::{json, ApiJson};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionsFilterRequest {
    #[serde(alias = "cpos")]
    pub operator_ids: Vec<uuid::Uuid>,
    #[serde(default, alias = "tariffs_ids")]
    pub tariff_ids: Vec<uuid::Uuid>,
    #[serde(default = "default_charging_modes")]
    pub charging_modes: Vec<ChargeType>,
}

fn default_charging_modes() -> Vec<ChargeType> {
    vec![ChargeType::AC, ChargeType::DC]
}

pub mod v3 {

    use super::*;

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ChargeConditionResponse {
        #[serde(serialize_with = "serialize_option_iso_8601")]
        pub last_updated_date: Option<chrono::DateTime<Utc>>,
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
        #[serde(skip)]
        pub updated: chrono::DateTime<Utc>,
    }

    type RequestConditionPath = Path<uuid::Uuid>;

    pub async fn get_handler(
        Extension(state): Extension<State>,
        path: Result<RequestConditionPath, PathRejection>,
    ) -> ApiJson<v3::ChargeConditionResponse> {
        let Path(operator_id) = path?;
        let response = charge_price::charge_conditions_standard(
            &mut *state.database_pool.acquire().await?,
            &[operator_id],
            &[],
            &[ChargeType::AC, ChargeType::DC],
        )
        .await?;

        if response.charging_conditions.is_empty() {
            return Err(ApiError::OperatorNotFound(operator_id.to_string()));
        }

        json(response)
    }

    pub async fn post_handler(
        Extension(state): Extension<State>,
        body: Result<Json<ConditionsFilterRequest>, JsonRejection>,
    ) -> ApiJson<v3::ChargeConditionResponse> {
        let Json(request) = body?;
        let response = charge_price::charge_conditions_custom(
            &mut *state.database_pool.acquire().await?,
            &request.operator_ids,
            &request.tariff_ids,
            &request.charging_modes,
        )
        .await?;

        json(response)
    }
}

pub mod v2 {

    use super::{v1::RequestCardPath, *};

    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ChargePriceMap<T> {
        pub operator: uuid::Uuid,
        pub ac: Vec<T>,
        pub dc: Vec<T>,
    }

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

    pub async fn get_handler(
        Extension(state): Extension<State>,
        path: RequestCardPath,
    ) -> ApiJsonList<v2::Card> {
        let Path((cpo_name, charge_type)) = path?;
        let cards = charge_price::get_cards(
            &mut *state.database_pool.acquire().await?,
            &charge_type,
            &cpo_name,
            &state.config.domain,
        )
        .await?;
        json_list(cards)
    }

    pub type AllCard<T> = Vec<ChargePriceMap<T>>;

    pub async fn post_handler(
        Extension(state): Extension<State>,
        body: Result<Json<ConditionsFilterRequest>, JsonRejection>,
    ) -> ApiJson<AllCard<v2::Card>> {
        let Json(request) = body?;
        get_all_cards_by_operator(&state, request).await
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

    pub type RequestCardPath = Result<Path<(String, ChargeType)>, PathRejection>;

    pub async fn get_handler(
        Extension(state): Extension<State>,
        path: RequestCardPath,
    ) -> ApiJsonList<v1::Card> {
        let Path((cpo_name, charge_type)) = path?;
        let cards = charge_price::get_cards::<_>(
            &mut *state.database_pool.acquire().await?,
            &charge_type,
            &cpo_name,
            &state.config.domain,
        )
        .await?;
        json_list(cards)
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

pub async fn get_all_cards_by_operator<T>(
    state: &State,
    request: ConditionsFilterRequest,
) -> ApiJson<v2::AllCard<T>>
where
    T: std::convert::From<crate::api::charge_condition::v2::Card>,
{
    let cards = charge_price::get_card_prices_by_operator(
        &mut *state.database_pool.acquire().await?,
        request.operator_ids,
        &state.config.domain,
        &request.tariff_ids,
    )
    .await?;
    json(cards)
}
