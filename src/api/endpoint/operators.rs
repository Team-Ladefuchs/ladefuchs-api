use axum::{
    extract::{rejection::PathRejection, Path, Query},
    Extension,
};

use crate::{
    api::{json, ApiJsonList},
    db::{
        self,
        operator::{Filter, Operator, OperatorV2, OperatorV3},
    },
    state::State,
};

use super::QueryFilter;

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
        Filter::Disabled => db::operator::disabled_operators_v1(&mut connection, &domain).await?,
    };

    json(operators)
}

pub async fn get_v2(
    Extension(state): Extension<State>,
    path: Result<Path<Filter>, PathRejection>,
) -> ApiJsonList<OperatorV2> {
    let Path(filter) = path?;
    let mut connection = state.database_pool.acquire().await?;
    let domain = &state.config.domain.to_string();
    let operators = match filter {
        Filter::All => db::operator::all_operators_v2(&mut connection, &domain).await?,
        Filter::Enabled => db::operator::enabled_operators_v2(&mut connection, &domain).await?,
        Filter::Disabled => db::operator::disabled_operators_v2(&mut connection, &domain).await?,
    };
    json(operators)
}

pub async fn get_v3(
    Extension(state): Extension<State>,
    filter: Query<QueryFilter>,
) -> ApiJsonList<OperatorV3> {
    let mut connection = state.database_pool.acquire().await?;
    let domain = &state.config.domain.to_string();
    let operators = if filter.standard {
        db::operator::enabled_operators_v3(&mut connection, &domain).await?
    } else {
        db::operator::all_operators_v3(&mut connection, &domain).await?
    };
    json(operators)
}
