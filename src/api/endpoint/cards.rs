use axum::{extract::Path, Extension, Json};

use crate::{
    api::{
        AllCard, ApiJson, ApiJsonList, CardByOperatorsAndTariffs, RequestCardPath,
        {json, json_list},
    },
    db::charge_price,
    state::State,
};

pub async fn get_all_cards_by_operator<T>(
    state: &State,
    request: CardByOperatorsAndTariffs,
) -> ApiJson<AllCard<T>>
where
    T: std::convert::From<crate::api::card::v3::Card>,
{
    let cards = charge_price::get_all_prices_by_cpo(
        &mut *state.database_pool.acquire().await?,
        request.operators,
        &state.config.domain,
        &request.tariffs,
    )
    .await?;
    json(cards)
}

pub mod v1 {
    use super::*;
    use crate::api::card::v1::Card;

    pub async fn cards(
        Extension(state): Extension<State>,
        path: RequestCardPath,
    ) -> ApiJsonList<v1::Card> {
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
}

pub mod v2 {
    use super::*;
    use crate::api::card::v2::Card;

    pub async fn cards(
        Extension(state): Extension<State>,
        path: RequestCardPath,
    ) -> ApiJsonList<v2::Card> {
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

    pub async fn card_by_operators_and_tariffs(
        Extension(state): Extension<State>,
        Json(request): Json<CardByOperatorsAndTariffs>,
    ) -> ApiJson<AllCard<v2::Card>> {
        get_all_cards_by_operator(&state, request).await
    }
}

pub mod v3 {
    use super::*;
    use crate::api::card::v3::Card;

    pub async fn cards(
        Extension(state): Extension<State>,
        path: RequestCardPath,
    ) -> ApiJsonList<v3::Card> {
        let Path((operator_name, charge_type)) = path?;
        let cards = charge_price::get(
            &mut *state.database_pool.acquire().await?,
            &charge_type,
            &operator_name,
            &state.config.domain,
        )
        .await?;
        json_list(cards)
    }

    pub async fn card_by_operators_and_tariffs(
        Extension(state): Extension<State>,
        Json(request): Json<CardByOperatorsAndTariffs>,
    ) -> ApiJson<AllCard<v3::Card>> {
        get_all_cards_by_operator(&state, request).await
    }
}
