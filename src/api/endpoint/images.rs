use crate::{api::{error::ApiError, util::json, ApiJsonList, card}, io::{FileStream, self}, state::State, db::{self, banner}};
use axum::{extract::Path, http::header, Extension};


pub async fn card_image(
    Extension(state): Extension<State>,
    Path(checksum): Path<String>,
) -> Result<(header::HeaderMap, FileStream), ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let image = db::card_image::get_by_checksum(&mut connection, &checksum)
        .await
        .map_err(|_| ApiError::NotFound)?;

    let stream = io::read_file_stream(&image.file_path).await?;
    Ok(stream)
}

pub async fn card_image_by_name(
    Path(image_name): Path<String>,
) -> Result<(header::HeaderMap, FileStream), ApiError> {
    let path = std::path::Path::new("./cards");
    let file = path.join(image_name);
    let stream = io::read_file_stream(&file).await?;
    Ok(stream)
}

pub async fn all_card_images(Extension(state): Extension<State>) -> ApiJsonList<card::Image> {
    let mut connection = state.database_pool.acquire().await?;
    let domain = &state.config.domain;
    let list = db::card_image::get_all(&mut connection, &domain).await?;

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
    Path(image_name): Path<String>,
) -> Result<(header::HeaderMap, FileStream), ApiError> {
    let path = std::path::Path::new(io::BANNER_PATH);
    let file = path.join(image_name);
    let resp = io::read_file_stream(&file).await?;

    Ok(resp)
}
