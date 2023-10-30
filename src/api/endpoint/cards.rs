use axum::{extract::Path, Extension, Json};

use crate::{
    api::{
        card, AllCard, ApiJson, ApiJsonList, CardByCpo, RequestCardPath, {json, json_list},
    },
    db::charge_price,
    state::State,
};

pub async fn card_by_operators(
    Extension(state): Extension<State>,
    Json(payload): Json<CardByCpo>,
) -> ApiJson<AllCard> {
    let cards = charge_price::get_all_prices_by_cpo(
        &mut *state.database_pool.acquire().await?,
        payload.cpos,
        &state.config.domain,
    )
    .await?;
    json(cards)
}

pub async fn cards_v1(
    Extension(state): Extension<State>,
    path: RequestCardPath,
) -> ApiJsonList<card::CardV1> {
    let Path((cpo_name, charge_type)) = path?;
    let cards = charge_price::get::<_>(
        &mut *state.database_pool.acquire().await?,
        &charge_type,
        &cpo_name,
        &state.config.domain,
    )
    .await?;
    json_list(cards)
}

pub async fn cards_v2(
    Extension(state): Extension<State>,
    path: RequestCardPath,
) -> ApiJsonList<card::CardV2> {
    let Path((cpo_name, charge_type)) = path?;
    let cards = charge_price::get(
        &mut *state.database_pool.acquire().await?,
        &charge_type,
        &cpo_name,
        &state.config.domain,
    )
    .await?;
    json_list(cards)
}
