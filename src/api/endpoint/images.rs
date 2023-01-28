use crate::{
    api::{error::ApiError, img, util::json, ApiJsonList},
    db::{self, banner},
    io::{self, FileStream},
    state::State,
};
use axum::{extract::Path, http::header, Extension};

pub async fn img_by_checksum(
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

pub async fn all_card_images(
    Extension(state): Extension<State>,
) -> ApiJsonList<img::TariffImage> {
    let mut connection = state.database_pool.acquire().await?;
    let domain = &state.config.domain;
    let list = db::image::get_all_cards(&mut connection, &domain).await?;

    json(list)
}

pub async fn all_cpo_images(Extension(state): Extension<State>) -> ApiJsonList<img::CpoImage> {
    let mut connection = state.database_pool.acquire().await?;
    let domain = &state.config.domain;
    let list = db::image::get_all_cpos(&mut connection, &domain).await?;

    json(list)
}

pub async fn get_affiliate_banners(
    Extension(state): Extension<State>,
) -> ApiJsonList<banner::Banner> {
    let mut connection = state.database_pool.acquire().await?;
    let list = banner::get_all_banner(&mut connection, &state.config.domain).await?;
    json(list)
}

pub async fn get_banner_image(
    Extension(state): Extension<State>,
    Path(image_id): Path<uuid::Uuid>,
) -> Result<(header::HeaderMap, FileStream), ApiError> {
    let mut connection = state.database_pool.acquire().await?;

    match banner::get_by_id(&mut connection, &image_id).await {
        Some((_, file_name)) => {
            let path = std::path::Path::new(io::BANNER_PATH);
            let file = path.join(file_name);
            let resp = io::read_file_stream(&file).await?;
            Ok(resp)
        }
        _ => Err(ApiError::NotFound),
    }
}
