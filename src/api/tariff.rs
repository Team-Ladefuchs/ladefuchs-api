use axum::{Extension, extract::Query};

use crate::api::ApiJson;
use crate::api::serialize_iso_8601;
use crate::{api::json, state::State};
use chrono::Utc;
use serde::{Deserialize, Serialize};

pub mod v3 {

    use axum::{Json, extract::rejection::JsonRejection};

    use super::*;
    use crate::ladefuchs_db;

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
        pub is_ad_hoc: bool,
    }

    fn is_zero(n: &f64) -> bool {
        n == &0.0
    }

    #[derive(Deserialize, Debug)]
    pub struct TariffQueryFilter {
        #[serde(default)]
        pub standard: bool,
    }

    fn default_true() -> bool {
        true
    }

    fn default_operator_ids() -> Vec<uuid::Uuid> {
        vec![]
    }

    pub async fn get_handler(
        Extension(state): Extension<State>,
        filter: Query<TariffQueryFilter>,
    ) -> ApiJson<v3::TariffResponse> {
        let mut connection = state.database_pool.acquire().await?;
        let tariffs = ladefuchs_db::tariff::v3::get_tariffs(
            &mut connection,
            &state.config.domain,
            filter.standard,
        )
        .await?;
        json(TariffResponse { tariffs })
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CustomTariffRequest {
        #[serde(default = "default_true")] // default just for beta
        pub standard: bool,
        pub add: Vec<uuid::Uuid>,
        pub remove: Vec<uuid::Uuid>,
        #[serde(default = "default_operator_ids")] // default just for beta
        pub operator_ids: Vec<uuid::Uuid>,
    }

    pub async fn post_handler(
        Extension(state): Extension<State>,
        request: Result<Json<CustomTariffRequest>, JsonRejection>,
    ) -> ApiJson<v3::TariffResponse> {
        let Json(payload) = request?;
        let mut connection = state.database_pool.acquire().await?;
        let tariffs = if payload.standard {
            ladefuchs_db::tariff::v3::get_standard_and_custom_with_operators(
                &mut connection,
                &state.config.domain,
                &payload.add,
                &payload.remove,
                &payload.operator_ids,
            )
            .await?
        } else {
            ladefuchs_db::tariff::v3::get_all_for_operators(
                &mut connection,
                &state.config.domain,
                &payload.operator_ids,
            )
            .await?
        };
        json(TariffResponse { tariffs })
    }
}

pub mod v4 {

    use super::*;
    use crate::ladefuchs_db;

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
        pub is_ad_hoc: bool,
        pub is_dynamic: bool,
    }

    fn is_zero(n: &f64) -> bool {
        n == &0.0
    }

    #[derive(Deserialize, Debug)]
    pub struct TariffQueryFilter {
        #[serde(default)]
        pub standard: bool,
    }

    pub async fn get_handler(
        Extension(state): Extension<State>,
        filter: Query<TariffQueryFilter>,
    ) -> ApiJson<v4::TariffResponse> {
        let mut connection = state.database_pool.acquire().await?;

        let tariffs = ladefuchs_db::tariff::v4::get_tariffs(
            &mut connection,
            &state.config.domain,
            filter.standard,
        )
        .await?;

        json(TariffResponse { tariffs })
    }
}
