use axum::{extract::Query, Extension};

use super::QueryFilter;
use crate::{
    api::{json, ApiJsonList},
    state::State,
};

pub mod v1 {
    use super::*;
    use crate::{api::tariff::v1::Tariff, db::tariff::v1::get_tariffs};

    pub async fn get_all(
        Extension(state): Extension<State>,
        filter: Query<QueryFilter>,
    ) -> ApiJsonList<v1::Tariff> {
        let mut connection = state.database_pool.acquire().await?;
        let tariffs = get_tariffs(&mut connection, &state.config.domain, filter.standard).await?;
        json(tariffs)
    }
}
