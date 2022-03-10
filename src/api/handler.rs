use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use axum::Json;

use reqwest::StatusCode;

use crate::db::card::{self, ChargeCardv1, ChargeCardv2, ChargeCardv3};
use crate::db::charging::ChargeType;
use crate::state::State;

type ApiJson<T> = Json<Vec<T>>;

pub async fn cards_v3(
    Extension(state): Extension<State>,
    // Path(cpo_name): Path<String>,
    Path((cpo_name, charge_type)): Path<(String, ChargeType)>,
) -> ApiJson<ChargeCardv3> {
    let cards = card::get_v3(&charge_type, &cpo_name, &state.database_pool)
        .await
        .unwrap();
    axum::Json(cards)
}

pub async fn cards_v2(
    Extension(state): Extension<State>,
    Path((cpo_name, charge_type)): Path<(String, ChargeType)>,
) -> ApiJson<ChargeCardv2> {
    let prices = card::get_v2(&charge_type, &cpo_name, &state.database_pool)
        .await
        .unwrap();
    axum::Json(prices)
}

pub async fn cards_v1(
    Extension(state): Extension<State>,
    Path((cpo_name, charge_type)): Path<(String, ChargeType)>,
) -> ApiJson<ChargeCardv1> {
    let prices = card::get_v1(&charge_type, &cpo_name, &state.database_pool)
        .await
        .unwrap();
    axum::Json(prices)
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Resource not found")
}
