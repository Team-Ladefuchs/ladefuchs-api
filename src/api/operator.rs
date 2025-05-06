use crate::ladefuchs_db::plug::ChargeType;
use crate::{
    api::{ApiJsonList, json},
    ladefuchs_db::{self, operator::Filter},
    state::State,
};
use axum::{
    Extension,
    extract::{Path, Query, rejection::PathRejection},
};
use chrono::Utc;
use chrono::serde::ts_seconds;

use serde::Serialize;

pub mod v1 {

    use super::*;
    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Operator {
        pub name: String,
        pub identifier: String,
        pub display_name: String,
    }

    pub async fn get_handler(
        Extension(state): Extension<State>,
        path: Result<axum::extract::Path<Filter>, PathRejection>,
    ) -> ApiJsonList<Operator> {
        let Path(filter) = path?;
        let mut connection = state.database_pool.acquire().await?;
        let domain = &state.config.domain.to_string();

        let operators = match filter {
            Filter::All => {
                ladefuchs_db::operator::all_operators_v1(&mut connection, domain).await?
            }
            Filter::Enabled => {
                ladefuchs_db::operator::enabled_operators_v1(&mut connection, domain).await?
            }
            Filter::Disabled => {
                ladefuchs_db::operator::disabled_operators_v1(&mut connection, domain).await?
            }
        };

        json(operators)
    }
}

pub mod v2 {

    use super::*;

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Operator {
        pub identifier: uuid::Uuid,
        pub display_name: String,
        pub types: Vec<ChargeType>,
        #[serde(with = "ts_seconds")]
        pub updated: chrono::DateTime<Utc>,
        pub image: Option<String>,
    }

    pub async fn get_handler(
        Extension(state): Extension<State>,
        path: Result<Path<Filter>, PathRejection>,
    ) -> ApiJsonList<Operator> {
        let Path(filter) = path?;
        let mut connection = state.database_pool.acquire().await?;
        let domain = &state.config.domain.to_string();
        let operators = match filter {
            Filter::All => {
                ladefuchs_db::operator::all_operators_v2(&mut connection, domain).await?
            }
            Filter::Enabled => {
                ladefuchs_db::operator::enabled_operators_v2(&mut connection, domain).await?
            }
            Filter::Disabled => {
                ladefuchs_db::operator::disabled_operators_v2(&mut connection, domain).await?
            }
        };
        json(operators)
    }
}

pub mod v3 {
    use axum::{Json, extract::rejection::JsonRejection};
    use serde::Deserialize;

    use crate::api::{ApiJson, serialize_option_iso_8601};

    use super::*;

    impl From<Vec<Operator>> for OperatorResponse {
        fn from(value: Vec<Operator>) -> Self {
            Self {
                last_updated_date: value.first().map(|item| item.updated),
                operators: value,
            }
        }
    }

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct OperatorResponse {
        #[serde(serialize_with = "serialize_option_iso_8601")]
        pub last_updated_date: Option<chrono::DateTime<Utc>>,
        pub operators: Vec<Operator>,
    }

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Operator {
        pub identifier: uuid::Uuid,
        pub name: String,
        pub charging_modes: Vec<ChargeType>,
        #[serde(skip)]
        pub updated: chrono::DateTime<Utc>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub image_url: Option<String>,
        pub is_standard: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub website_url: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    pub struct OperatorQueryFilter {
        #[serde(default)]
        pub standard: bool,
    }
    pub async fn get_handler(
        Extension(state): Extension<State>,
        filter: Query<OperatorQueryFilter>,
    ) -> ApiJson<OperatorResponse> {
        let mut connection = state.database_pool.acquire().await?;
        let domain = &state.config.domain.to_string();
        let operators = if filter.standard {
            ladefuchs_db::operator::enabled_operators_v3(&mut connection, domain).await?
        } else {
            ladefuchs_db::operator::all_operators_v3(&mut connection, domain).await?
        };
        json(OperatorResponse::from(operators))
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CustomOperatorRequest {
        pub add: Vec<uuid::Uuid>,
        pub remove: Vec<uuid::Uuid>,
    }

    pub async fn post_handler(
        Extension(state): Extension<State>,
        request: Result<Json<CustomOperatorRequest>, JsonRejection>,
    ) -> ApiJson<OperatorResponse> {
        let mut connection = state.database_pool.acquire().await?;
        let Json(payload) = request?;

        let operators = ladefuchs_db::operator::get_custom_operators(
            &mut connection,
            &state.config.domain,
            &payload.add,
            &payload.remove,
        )
        .await?;
        json(OperatorResponse::from(operators))
    }
}
