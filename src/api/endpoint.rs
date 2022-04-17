use axum::extract::rejection::PathRejection;
use axum::extract::{Extension, Path};
use axum::response::IntoResponse;

use reqwest::StatusCode;

use crate::db::card::{self};
use crate::state::State;

use super::operator::{self, Mode};
use super::util::{json, json_list};
use super::{ApiJsonList, RequestCardPath};

pub async fn cards_v1(
    Extension(state): Extension<State>,
    path: RequestCardPath,
) -> ApiJsonList<card::CardV1> {
    let Path((cpo_name, charge_type)) = path?;
    let cards = card::get_v1(&charge_type, &cpo_name, &state.database_pool).await?;
    json_list(cards)
}

pub async fn cards_v2(
    Extension(state): Extension<State>,
    path: RequestCardPath,
) -> ApiJsonList<card::CardV2> {
    let Path((cpo_name, charge_type)) = path?;
    let cards = card::get_with_ioniq::<_>(&charge_type, &cpo_name, &state.database_pool).await?;
    json_list(cards)
}

pub async fn cards_v3(
    Extension(state): Extension<State>,
    path: RequestCardPath,
) -> ApiJsonList<card::CardV3> {
    let Path((cpo_name, charge_type)) = path?;
    let cards = card::get_with_ioniq(&charge_type, &cpo_name, &state.database_pool).await?;
    json_list(cards)
}

pub async fn operators(
    Extension(state): Extension<State>,
    path: Result<Path<Mode>, PathRejection>,
) -> ApiJsonList<operator::Operator> {
    let Path(filter) = path?;
    let operators = operator::get_operators(filter, &state.database_pool).await?;

    json(operators)
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Resource not found")
}
