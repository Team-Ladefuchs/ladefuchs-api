use crate::api::ApiJson;
use crate::state::State;

// use crate::api::json;
// use crate::db::banner;
use axum::Extension;
// use rand::prelude::*;

pub mod v3 {

    use rand::{rng, seq::IndexedRandom};

    use crate::{
        api::{self, json},
        ladefuchs_db::banner,
    };

    use super::*;

    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CPriceBannerAd {
        pub image_url: url::Url,
        pub affiliate_link_url: url::Url,
    }

    pub async fn get_handler(Extension(state): Extension<State>) -> ApiJson<CPriceBannerAd> {
        let mut connection = state.database_pool.acquire().await?;
        let result = match banner::v2::get_all_banner(
            &mut connection,
            &state.config.domain,
            banner::BannerPathVersion::V3,
            Some(banner::BannerStatus::Active),
        )
        .await?
        .choose(&mut rng())
        {
            Some(random_banner) => CPriceBannerAd {
                image_url: random_banner.image.clone(),
                affiliate_link_url: random_banner.link.clone(),
            },
            None => {
                return Err(api::ApiError::General(eyre::Error::msg(
                    "could not serve a fallback banner",
                )));
            }
        };
        json(result)
    }
}
