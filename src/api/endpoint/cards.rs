use axum::{Extension, extract::Path};

use crate::{api::{ApiJsonList, RequestCardPath, util::json_list, card}, state::State, db::charge_price};

pub async fn cards_v1(
    Extension(state): Extension<State>,
    path: RequestCardPath,
) -> ApiJsonList<card::CardV1> {
    let Path((cpo_name, charge_type)) = path?;
    let cards = charge_price::get::<_>(
        &mut state.database_pool.acquire().await?,
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
        &mut state.database_pool.acquire().await?,
        &charge_type,
        &cpo_name,
        &state.config.domain,
    )
    .await?;
    json_list(cards)
}
