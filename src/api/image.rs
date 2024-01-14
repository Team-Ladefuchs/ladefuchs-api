use crate::{
    api::{error::ApiError, image, json, ApiJsonList},
    db::{self},
    io::{self},
    state::State,
};
use axum::{body::Body, extract::Path, http::header, Extension};
use chrono::Utc;
use serde::Serialize;

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

#[cfg(test)]
mod tests {
    use axum::{
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    use crate::db::image::Image;

    use super::*;

    #[tokio::test]
    async fn get_image_by_checksum_success() {
        // given
        // TODO: create a Database trait
        // TODO: add `mockall::automock` as cfg_attr
        let mut mock_db = MockDatabase::new();
        mock_db.expect_get_by_checksum().return_once(|_| {
            Ok(Image {
                checksum: 1,
                file_path: "".into(),
                mime: mime::IMAGE_JPEG,
            })
        });
        let state: Extension<State> = State::new(db_pool, config, timer);
        let app = Router::new().nest("/v3/images", get(image::v3::get_handler(&state).await));

        // when
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v3/images")
                    .method("GET")
                    .header("Authorization", "Fake")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // then
        assert_eq!(response.status(), StatusCode::OK);

        // TODO: assert body
        //let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        //let body: String = serde_json::from_slice(&body).unwrap();
    }
}
