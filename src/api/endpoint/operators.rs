use axum::{
    extract::{rejection::PathRejection, Path},
    Extension,
};

use crate::{
    api::{json, ApiJsonList},
    db::{
        self,
        cpo::{Filter, Operator, OperatorV2},
    },
    state::State,
};

pub async fn get(
    Extension(state): Extension<State>,
    path: Result<axum::extract::Path<Filter>, PathRejection>,
) -> ApiJsonList<Operator> {
    let Path(filter) = path?;
    let mut connection = state.database_pool.acquire().await?;
    let domain = &state.config.domain.to_string();

    let operators = match filter {
        Filter::All => db::cpo::all_operators_v1(&mut connection, &domain).await?,
        Filter::Enabled => db::cpo::enabled_operators_v1(&mut connection, &domain).await?,
        Filter::Disabled => db::cpo::disabled_operators_v1(&mut connection, &domain).await?,
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
        Filter::All => db::cpo::all_operators_v2(&mut connection, &domain).await?,
        Filter::Enabled => db::cpo::enabled_operators_v2(&mut connection, &domain).await?,
        Filter::Disabled => db::cpo::disabled_operators_v2(&mut connection, &domain).await?,
    };
    json(operators)
}
