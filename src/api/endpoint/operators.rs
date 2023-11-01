use super::QueryFilter;

use crate::db::plug::ChargeType;
use crate::{
    api::{json, ApiJsonList},
    db::{self, operator::Filter},
    state::State,
};
use axum::{
    extract::{rejection::PathRejection, Path, Query},
    Extension,
};
use chrono::serde::ts_seconds;
use chrono::Utc;

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

    pub async fn get(
        Extension(state): Extension<State>,
        path: Result<axum::extract::Path<Filter>, PathRejection>,
    ) -> ApiJsonList<Operator> {
        let Path(filter) = path?;
        let mut connection = state.database_pool.acquire().await?;
        let domain = &state.config.domain.to_string();

        let operators = match filter {
            Filter::All => db::operator::all_operators_v1(&mut connection, &domain).await?,
            Filter::Enabled => db::operator::enabled_operators_v1(&mut connection, &domain).await?,
            Filter::Disabled => {
                db::operator::disabled_operators_v1(&mut connection, &domain).await?
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

    pub async fn get(
        Extension(state): Extension<State>,
        path: Result<Path<Filter>, PathRejection>,
    ) -> ApiJsonList<Operator> {
        let Path(filter) = path?;
        let mut connection = state.database_pool.acquire().await?;
        let domain = &state.config.domain.to_string();
        let operators = match filter {
            Filter::All => db::operator::all_operators_v2(&mut connection, &domain).await?,
            Filter::Enabled => db::operator::enabled_operators_v2(&mut connection, &domain).await?,
            Filter::Disabled => {
                db::operator::disabled_operators_v2(&mut connection, &domain).await?
            }
        };
        json(operators)
    }
}

pub mod v3 {
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
        pub standard: bool,
        pub url: Option<String>,
    }

    pub async fn get(
        Extension(state): Extension<State>,
        filter: Query<QueryFilter>,
    ) -> ApiJsonList<Operator> {
        let mut connection = state.database_pool.acquire().await?;
        let domain = &state.config.domain.to_string();
        let operators = if filter.standard {
            db::operator::enabled_operators_v3(&mut connection, &domain).await?
        } else {
            db::operator::all_operators_v3(&mut connection, &domain).await?
        };
        json(operators)
    }
}
