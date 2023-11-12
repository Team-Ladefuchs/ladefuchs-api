use crate::{
    api::{error::ApiError, image, json, ApiJsonList},
    db::{self, banner},
    io::{self, FileStream},
    state::State,
};
use axum::{extract::Path, http::header, Extension};

pub async fn image_by_checksum(
    Extension(state): Extension<State>,
    Path(checksum): Path<String>,
) -> Result<(header::HeaderMap, FileStream), ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let image = db::image::get_by_checksum(&mut connection, &checksum)
        .await
        .map_err(|_| ApiError::NotFound)?;

    let stream = io::read_file_stream(&image.file_path).await?;
    Ok(stream)
}

pub mod v2 {
    use crate::db::banner::BannerPathVersion;

    use super::*;

    pub async fn all_banners(
        Extension(state): Extension<State>,
    ) -> ApiJsonList<banner::v2::Banner> {
        let mut connection = state.database_pool.acquire().await?;
        let list = banner::v2::get_all_banner(
            &mut connection,
            &state.config.domain,
            BannerPathVersion::V2,
        )
        .await?;
        json(list)
    }

    pub async fn all_card_images(
        Extension(state): Extension<State>,
    ) -> ApiJsonList<image::v2::TariffImage> {
        let mut connection = state.database_pool.acquire().await?;
        let domain = &state.config.domain;
        let list = db::image::v2::get_all_tariffs(&mut connection, &domain).await?;

        json(list)
    }

    pub async fn all_cpo_images(
        Extension(state): Extension<State>,
    ) -> ApiJsonList<image::v2::OperatorImage> {
        let mut connection = state.database_pool.acquire().await?;
        let domain = &state.config.domain;
        let list = db::image::v2::get_all_operators(&mut connection, &domain).await?;

        json(list)
    }
}

pub mod v3 {
    use crate::db::banner::BannerPathVersion;

    use super::*;

    pub async fn all_banners(
        Extension(state): Extension<State>,
    ) -> ApiJsonList<banner::v3::Banner> {
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

    pub async fn all_images(
        Extension(state): Extension<State>,
    ) -> ApiJsonList<image::v3::GenericImage> {
        let mut connection = state.database_pool.acquire().await?;
        let domain = &state.config.domain;
        let list = db::image::v3::get_all(&mut connection, &domain).await?;

        json(list)
    }
}
