use crate::api::json;
use crate::db::banner;
use crate::db::banner::BannerPathVersion;
use crate::{api::ApiJsonList, state::State};

use axum::Extension;

pub mod v2 {
    use super::*;

    pub async fn banner(Extension(state): Extension<State>) -> ApiJsonList<banner::v2::Banner> {
        let mut connection = state.database_pool.acquire().await?;
        let list = banner::v2::get_all_banner(
            &mut connection,
            &state.config.domain,
            BannerPathVersion::V2,
        )
        .await?;
        json(list)
    }
}

pub mod v3 {
    use super::*;

    pub async fn banners(Extension(state): Extension<State>) -> ApiJsonList<banner::v3::Banner> {
        let mut connection = state.database_pool.acquire().await?;
        let list = banner::v2::get_all_banner(
            &mut connection,
            &state.config.domain,
            BannerPathVersion::V3,
        )
        .await?
        .into_iter()
        .map(|banner| banner.into())
        .collect();
        json(list)
    }
}
