use crate::api::json;
use crate::db::banner;
use crate::db::banner::BannerPathVersion;
use crate::{api::ApiJsonList, state::State};
use axum::Extension;
use chrono::Utc;
use serde::Serialize;

pub mod v2 {
    use chrono::serde::ts_seconds;

    use super::*;

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Banner {
        pub link: url::Url,
        pub image: url::Url,
        pub frequency: i16,
        pub is_affiliate: bool,
        pub id: uuid::Uuid,
        #[serde(with = "ts_seconds")]
        pub updated: chrono::DateTime<Utc>,
        pub filename: String,
    }

    pub async fn get_handler(Extension(state): Extension<State>) -> ApiJsonList<Banner> {
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
    use axum::Json;
    use banner::PlatformType;
    use serde::Deserialize;

    use crate::{api::error::ApiError, db};

    use self::v3::Banner;

    use super::*;
    pub mod v3 {
        use super::*;
        use crate::api::serialize_iso_8601;

        impl From<v2::Banner> for Banner {
            fn from(value: v2::Banner) -> Self {
                Self {
                    affiliate_link_url: value.link,
                    image_url: value.image,
                    frequency: value.frequency,
                    is_affiliate: value.is_affiliate,
                    identifier: value.id,
                    last_updated_date: value.updated,
                }
            }
        }

        #[derive(Serialize, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct Banner {
            pub affiliate_link_url: url::Url,
            pub identifier: uuid::Uuid,
            pub image_url: url::Url,
            pub frequency: i16,
            pub is_affiliate: bool,
            #[serde(serialize_with = "serialize_iso_8601")]
            pub last_updated_date: chrono::DateTime<Utc>,
        }
    }

    pub async fn get_handler(Extension(state): Extension<State>) -> ApiJsonList<Banner> {
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

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]

    pub struct ImpressionBannerRequest {
        pub banner_id: i32,
        pub plattform: PlatformType,
    }

    pub async fn post_impression_handler(
        Extension(state): Extension<State>,
        Json(request): Json<ImpressionBannerRequest>,
    ) -> Result<(), ApiError> {
        let mut connection = state.database_pool.acquire().await?;

        db::banner::add_banner_impression(&mut connection, &request.banner_id, &request.plattform)
            .await?;
        Ok(())
    }
}
