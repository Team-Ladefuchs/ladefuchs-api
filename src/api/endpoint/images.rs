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
pub mod v3 {

    use super::*;

    pub async fn images(
        Extension(state): Extension<State>,
    ) -> ApiJsonList<image::v3::GenericImage> {
        let mut connection = state.database_pool.acquire().await?;
        let domain = &state.config.domain;
        let list = db::image::v3::get_all(&mut connection, &domain).await?;

        json(list)
    }
}
