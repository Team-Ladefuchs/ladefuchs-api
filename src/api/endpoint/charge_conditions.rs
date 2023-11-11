use axum::{extract::Path, Extension, Json};

use crate::{
    api::{
        AllCard, ApiJson, ApiJsonList, ConditionsFilterRequest, {json, json_list},
    },
    db::charge_price,
    state::State,
};

pub async fn get_all_cards_by_operator<T>(
    state: &State,
    request: ConditionsFilterRequest,
) -> ApiJson<AllCard<T>>
where
    T: std::convert::From<crate::api::charge_conditions::v2::Card>,
{
    let cards = charge_price::get_card_prices_by_operator(
        &mut *state.database_pool.acquire().await?,
        request.operator_ids,
        &state.config.domain,
        &request.tariff_ids,
    )
    .await?;
    json(cards)
}

pub mod v1 {
    use super::*;
    use crate::api::charge_conditions::v1::Card;

    pub async fn cards(
        Extension(state): Extension<State>,
        path: crate::api::RequestCardPath,
    ) -> ApiJsonList<v1::Card> {
        let Path((cpo_name, charge_type)) = path?;
        let cards = charge_price::get_cards::<_>(
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
    use crate::api::{charge_conditions::v2::Card, RequestCardPath};

    pub async fn cards(
        Extension(state): Extension<State>,
        path: RequestCardPath,
    ) -> ApiJsonList<v2::Card> {
        let Path((cpo_name, charge_type)) = path?;
        let cards = charge_price::get_cards(
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
        Json(request): Json<ConditionsFilterRequest>,
    ) -> ApiJson<AllCard<v2::Card>> {
        get_all_cards_by_operator(&state, request).await
    }
}

pub mod v3 {
    use super::*;
    use crate::{
        api::{self, charge_conditions::v3, error::ApiError},
        db::plug::ChargeType,
    };

    pub async fn charge_conditions(
        Extension(state): Extension<State>,
        path: api::RequestConditionPath,
    ) -> ApiJson<v3::ChargeConditionResponse> {
        let Path(operator_id) = path?;
        let response = charge_price::get_charge_conditions(
            &mut *state.database_pool.acquire().await?,
            &[operator_id],
            &[],
            &[ChargeType::AC, ChargeType::DC],
        )
        .await?;

        if response.charging_conditions.is_empty() {
            return Err(ApiError::OperatorNotFound(operator_id.to_string()));
        }

        json(response)
    }

    pub async fn charge_conditions_with_filter(
        Extension(state): Extension<State>,
        Json(request): Json<api::ConditionsFilterRequest>,
    ) -> ApiJson<v3::ChargeConditionResponse> {
        let response = charge_price::get_charge_conditions(
            &mut *state.database_pool.acquire().await?,
            &request.operator_ids,
            &request.operator_ids,
            &request.charging_modes,
        )
        .await?;

        json(response)
    }
}
