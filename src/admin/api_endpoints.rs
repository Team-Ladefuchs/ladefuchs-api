use axum::{
    Extension,
    extract::{Json, Multipart},
};
use sqlx::Acquire;
use std::path::{Path as FilePath, PathBuf};

use crate::{
    api::{
        ApiJson,
        error::{self, ApiError},
        json,
    },
    io,
    ladefuchs_db::{image, operator::admin, tariff},
    slack::{self, Emoji, SlackClient},
    state::State,
};

pub async fn post_image(
    Extension(state): Extension<State>,
    mut multipart: Multipart,
) -> ApiJson<i32> {
    let mut upload = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| ApiError::General(error.into()))?
    {
        if field.name() != Some("image") {
            continue;
        }

        let supplied_filename = field
            .file_name()
            .ok_or_else(|| ApiError::General(eyre::eyre!("Image has no filename")))?;
        let filename = FilePath::new(supplied_filename)
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty() && *name != "." && *name != "..")
            .ok_or_else(|| ApiError::General(eyre::eyre!("Image has an invalid filename")))?
            .to_owned();
        if filename != supplied_filename || filename.contains('\\') {
            return Err(ApiError::General(eyre::eyre!(
                "Image filename must not contain a path"
            )));
        }
        let bytes = field
            .bytes()
            .await
            .map_err(|error| ApiError::General(error.into()))?;

        upload = Some((filename, bytes));
        break;
    }

    let (filename, bytes) =
        upload.ok_or_else(|| ApiError::General(eyre::eyre!("Missing image field")))?;
    let mime = io::guess_image_mime_bytes(&bytes).map_err(|mime| {
        ApiError::General(eyre::eyre!(
            "Unsupported image type: {mime}. Expected JPEG, PNG, GIF, or SVG."
        ))
    })?;
    let image_path = PathBuf::from(io::IMAGE_UPLOAD_PATH).join(filename);
    let checksum = blake3::hash(&bytes).to_hex().to_string();

    tokio::fs::write(&image_path, &bytes)
        .await
        .map_err(|error| ApiError::General(error.into()))?;
    let updated = tokio::fs::metadata(&image_path)
        .await
        .map_err(|error| ApiError::General(error.into()))?
        .modified()
        .map_err(|error| ApiError::General(error.into()))?
        .into();

    let mut connection = state.database_pool.acquire().await?;
    let mut transaction = connection.begin().await?;
    let image_id = image::insert_or_update(
        &mut transaction,
        &image::ImageContext {
            image: image::Image {
                file_path: image_path,
                checksum,
                mime,
            },
            updated,
        },
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    transaction.commit().await?;

    json(image_id)
}

pub async fn patch_operator(
    Extension(state): Extension<State>,
    Json(mut operator): Json<admin::Operator>,
) -> Result<ApiJson<admin::Operator>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let mut transaction: sqlx::Transaction<sqlx::Postgres> = connection.begin().await?;
    operator.update(&mut transaction).await?;

    transaction.commit().await?;

    match operator.image {
        Some(image_id) => {
            if let Err(error) = image::update_image_file_name(
                &mut connection,
                &operator.name,
                image_id,
                Some("cpo"),
            )
            .await
            {
                tracing::error!(
                    operator_id = operator.id,
                    internal_name = operator.name,
                    slug_name = operator.slug_name,
                    image_id = image_id,
                    %error,
                    "Could update internal operator name",
                )
            }
        }
        None => {
            let slack = &state.slack;
            let url_str = operator.url.as_deref().unwrap_or_default();
            let msg = format!(
                "Hi {},this CPO {:#?} has no image.\nI have some useful information:\nName Internal: {}\n{}",
                slack::MALIK,
                operator.slug_name,
                operator.name,
                url_str
            );
            slack
                .send_message(slack::TextMessage {
                    emoji: Some(Emoji::ElectricPlug),
                    text: msg,
                    markdown: false,
                })
                .await;
        }
    }

    Ok(json(operator))
}

pub async fn patch_tariff(
    Extension(state): Extension<State>,
    Json(payload): Json<tariff::admin::UpdateTariffInternal>,
) -> Result<(), error::ApiError> {
    let mut connection: sqlx::pool::PoolConnection<sqlx::Postgres> =
        state.database_pool.acquire().await?;
    let mut transaction: sqlx::Transaction<'_, sqlx::Postgres> = connection.begin().await?;
    tariff::admin::update_partial(&mut transaction, &payload).await?;
    transaction.commit().await?;

    Ok(())
}
