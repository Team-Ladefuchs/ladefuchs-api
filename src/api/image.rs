use crate::{
    api::{error::ApiError, image, json, ApiJsonList},
    db::{self},
    io::{self},
    state::State,
};
use axum::{
    body::Body,
    extract::{Path, Query},
    http::header,
    Extension,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::serialize_iso_8601;

pub async fn image_by_checksum(
    Extension(state): Extension<State>,
    Path(checksum): Path<String>,
) -> Result<(header::HeaderMap, Body), ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let image = db::image::get_by_checksum(&mut connection, &checksum)
        .await
        .map_err(|_| ApiError::NotFound)?;

    let stream = io::read_file_stream(&image.file_path).await?;
    Ok(stream)
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageProxyQuery {
    image: url::Url,
}

pub async fn image_proxy(
    Extension(state): Extension<State>,
    Query(query): Query<ImageProxyQuery>,
) -> Result<Body, ApiError> {
    let bytes = state
        .http_client
        .get(query.image)
        .send()
        .await
        .map_err(|e| eyre::Error::new(e))?
        .bytes()
        .await
        .map_err(|e| super::ApiError::General(e.into()))?;

    Ok(Body::from(bytes))
}

pub mod v3 {
    use super::*;

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub enum RelationType {
        Tariff,
        Operator,
        Banner,
    }

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GenericImage {
        pub relation_id: uuid::Uuid,
        pub relation_type: RelationType,
        pub blake3sum: String,
        #[serde(serialize_with = "serialize_iso_8601")]
        pub last_updated_date: chrono::DateTime<Utc>,
        pub image_url: url::Url,
    }

    pub async fn get_handler(
        Extension(state): Extension<State>,
    ) -> ApiJsonList<image::v3::GenericImage> {
        let mut connection = state.database_pool.acquire().await?;
        let domain = &state.config.domain;
        let list = db::image::v3::get_all(&mut connection, &domain).await?;

        json(list)
    }
}
