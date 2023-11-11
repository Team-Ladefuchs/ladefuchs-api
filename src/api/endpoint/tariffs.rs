use axum::{extract::Query, Extension};

use super::QueryFilter;
use crate::{api::json, state::State};

pub mod v3 {
    use super::*;
    use crate::{
        api::{tariff::v3::TariffResponse, ApiJson},
        db::tariff::v1::get_tariffs,
    };

    pub async fn get_all(
        Extension(state): Extension<State>,
        filter: Query<QueryFilter>,
    ) -> ApiJson<v3::TariffResponse> {
        let mut connection = state.database_pool.acquire().await?;
        let tariffs = get_tariffs(&mut connection, &state.config.domain, filter.standard).await?;
        json(TariffResponse { tariffs })
    }
}
