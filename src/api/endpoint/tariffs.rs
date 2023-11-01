use axum::{extract::Query, Extension};

use super::QueryFilter;
use crate::{
    api::{json, ApiJsonList},
    db::tariff::{self},
    state::State,
};

pub mod v1 {

    use super::*;
    use crate::api::tariff::v1::Tariff;

    pub async fn get_all(
        Extension(state): Extension<State>,
        filter: Query<QueryFilter>,
    ) -> ApiJsonList<v1::Tariff> {
        let mut connection = state.database_pool.acquire().await?;
        let tariffs =
            tariff::get_tariffs_v1(&mut connection, &state.config.domain, filter.standard).await?;
        json(tariffs)
    }
}
