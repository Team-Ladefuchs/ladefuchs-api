use axum::extract::rejection::PathRejection;
use axum::extract::{Extension, Path};
use axum::response::IntoResponse;

use reqwest::StatusCode;

use crate::db::{self, charge_price};
use crate::state::State;

use super::card;
use super::operator::{self, Filter};
use super::util::{json, json_list};
use super::{ApiJsonList, RequestCardPath};

pub async fn cards_v1(
    Extension(state): Extension<State>,
    path: RequestCardPath,
) -> ApiJsonList<card::CardV1> {
    let Path((cpo_name, charge_type)) = path?;
    let cards = charge_price::get_v1(
        &mut state.database_pool.acquire().await?,
        &charge_type,
        &cpo_name,
    )
    .await?;
    json_list(cards)
}

pub async fn cards_v2(
    Extension(state): Extension<State>,
    path: RequestCardPath,
) -> ApiJsonList<card::CardV2> {
    let Path((cpo_name, charge_type)) = path?;
    let cards = charge_price::get_with_ioniq::<_>(
        &mut state.database_pool.acquire().await?,
        &charge_type,
        &cpo_name,
    )
    .await?;
    json_list(cards)
}

pub async fn cards_v3(
    Extension(state): Extension<State>,
    path: RequestCardPath,
) -> ApiJsonList<card::CardV3> {
    let Path((cpo_name, charge_type)) = path?;
    let cards = charge_price::get_with_ioniq(
        &mut state.database_pool.acquire().await?,
        &charge_type,
        &cpo_name,
    )
    .await?;
    json_list(cards)
}

pub async fn operators(
    Extension(state): Extension<State>,
    path: Result<Path<Filter>, PathRejection>,
) -> ApiJsonList<operator::Operator> {
    let Path(filter) = path?;
    let operators =
        db::cpo::get_operators(&mut state.database_pool.acquire().await?, filter).await?;

    json(operators)
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Resource not found")
}
