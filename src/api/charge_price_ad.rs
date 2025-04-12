use crate::api::ApiJson;
use crate::state::State;

use crate::api::json;
use crate::db::banner;
use axum::Extension;
use rand::prelude::*;

pub mod v3 {

    use axum::debug_handler;
    use rand::rng;

    use crate::api;

    use super::*;

    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ChargePriceBannerAd {
        pub image_url: url::Url,
        pub affiliate_link_url: url::Url,
    }
    #[debug_handler]
    pub async fn get_handler(Extension(state): Extension<State>) -> ApiJson<ChargePriceBannerAd> {
        // let banner = match state.charge_price_api.fetch_advertisements().await {
        //     Ok(charge_price_ad) => {
        //         let mut image_url = state.config.domain.clone();
        //         image_url.set_path("/image/proxy");
        //         image_url
        //             .query_pairs_mut()
        //             .append_pair("image", &charge_price_ad.banner_image_url.to_string());
        //         ChargePriceBannerAd {
        //             image_url,
        //             affiliate_link_url: charge_price_ad.cta_url,
        //         }
        //     }
        //     Err(error) => {
        //         tracing::warn!(context = "Chargeprice ad handler", %error);
        //         let mut connection = state.database_pool.acquire().await?;
        //         match banner::v2::get_all_banner(
        //             &mut connection,
        //             &state.config.domain,
        //             banner::BannerPathVersion::V3,
        //         )
        //         .await?
        //         .choose(&mut rng())
        //         {
        //             Some(random_banner) => ChargePriceBannerAd {
        //                 image_url: random_banner.image.clone(),
        //                 affiliate_link_url: random_banner.link.clone(),
        //             },
        //             None => {
        //                 return Err(api::ApiError::General(eyre::Error::msg(
        //                     "could not serve a fallback banner",
        //                 )))
        //             }
        //         }
        //     }
        // };
        // json(banner)
        todo!()
    }
}
