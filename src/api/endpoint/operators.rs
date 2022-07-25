use axum::{
    extract::{rejection::PathRejection, Path},
    Extension,
};

use crate::{
    api::{
        operator::{self, Filter, Operator},
        util::json,
        ApiJsonList,
    },
    db,
    state::State,
};

pub async fn get(
    Extension(state): Extension<State>,
    path: Result<axum::extract::Path<Filter>, PathRejection>,
) -> ApiJsonList<Operator> {
    let Path(filter) = path?;
    let operators = db::cpo::get_operators::<operator::Operator>(
        &mut state.database_pool.acquire().await?,
        filter,
    )
    .await?;

    json(operators)
}

pub async fn get_v2(
    Extension(state): Extension<State>,
    path: Result<Path<Filter>, PathRejection>,
) -> ApiJsonList<operator::OperatorV2> {
    let Path(filter) = path?;
    let operators = db::cpo::get_operators::<operator::OperatorV2>(
        &mut state.database_pool.acquire().await?,
        filter,
    )
    .await?;
    json(operators)
}
