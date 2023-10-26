use axum::{extract::Query, Extension};

use super::QueryFilter;
use crate::{
    api::{json, ApiJsonList},
    db::tariff::{self, TariffV1},
    state::State,
};

pub async fn get_v1(
    Extension(state): Extension<State>,
    filter: Query<QueryFilter>,
) -> ApiJsonList<TariffV1> {
    let mut connection = state.database_pool.acquire().await?;
    let tariffs =
        tariff::get_tariffs_v1(&mut connection, &state.config.domain, filter.standard).await?;
    json(tariffs)
}
