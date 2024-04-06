use axum::{extract::Query, Extension};

use crate::api::serialize_iso_8601;
use crate::api::ApiJson;
use crate::{api::json, state::State};
use chrono::Utc;
use serde::{Deserialize, Serialize};

pub mod v3 {

    use crate::{api::OperatorQueryFilter, db};

    use super::*;

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

    pub async fn get_handler(
        Extension(state): Extension<State>,
        filter: Query<OperatorQueryFilter>,
    ) -> ApiJson<v3::TariffResponse> {
        let mut connection = state.database_pool.acquire().await?;
        let tariffs =
            db::tariff::v3::get_tariffs(&mut connection, &state.config.domain, filter.standard)
                .await?;
        json(TariffResponse { tariffs })
    }
}
