use super::card;
use super::error::ApiError;
use super::operator::{self, Filter};
use super::util::{json, json_list};
use super::{ApiJsonList, RequestCardPath};
use crate::db::{self, charge_price};
use crate::image::{self, FileStream};
use crate::state::State;
use axum::extract::rejection::PathRejection;
use axum::extract::{Extension, Path};
use axum::http::{header, HeaderValue};
use std::borrow::Cow::Borrowed;

pub async fn cards_v1(
    Extension(state): Extension<State>,
    path: RequestCardPath,
) -> ApiJsonList<card::CardV1> {
    let Path((cpo_name, charge_type)) = path?;
    let cards = charge_price::get::<_>(
        &mut state.database_pool.acquire().await?,
        &charge_type,
        &cpo_name,
        &state.config.domain,
    )
    .await?;
    json_list(cards)
}

pub async fn cards_v2(
    Extension(state): Extension<State>,
    path: RequestCardPath,
) -> ApiJsonList<card::CardV2> {
    let Path((cpo_name, charge_type)) = path?;
    let cards = charge_price::get::<_>(
        &mut state.database_pool.acquire().await?,
        &charge_type,
        &cpo_name,
        &state.config.domain,
    )
    .await?;
    json_list(cards)
}

pub async fn cards_v3(
    Extension(state): Extension<State>,
    path: RequestCardPath,
) -> ApiJsonList<card::CardV3> {
    let Path((cpo_name, charge_type)) = path?;
    let cards = charge_price::get(
        &mut state.database_pool.acquire().await?,
        &charge_type,
        &cpo_name,
        &state.config.domain,
    )
    .await?;
    json_list(cards)
}

pub async fn operators(
    Extension(state): Extension<State>,
    path: Result<Path<Filter>, PathRejection>,
) -> ApiJsonList<operator::Operator> {
    let Path(filter) = path?;
    dbg!(&filter);
    let operators =
        db::cpo::get_operators(&mut state.database_pool.acquire().await?, filter).await?;

    json(operators)
}

pub async fn handler_404() -> ApiError {
    ApiError::NotFound
}

pub async fn card_image(
    Extension(state): Extension<State>,
    Path(path): Path<String>,
) -> Result<(header::HeaderMap, FileStream), ApiError> {
    let checksum = parse_file_parameter(&path)?;
    let mut connection = state.database_pool.acquire().await?;
    let image = db::card_image::get_by_checksum(&mut connection, &checksum)
        .await
        .map_err(|_| ApiError::NotFound)?;

    let body = image::read_file(&image.file_path).await?;

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, image.mime.as_ref().try_into()?);
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!(
            "attachment; filename=\"{}\"",
            image
                .file_path
                .file_name()
                .map(|f| f.to_string_lossy())
                .unwrap_or_else(|| Borrowed("unknown_file"))
        )
        .parse()?,
    );
    let resp = (headers, body);
    Ok(resp)
}

fn parse_file_parameter(path: &str) -> Result<String, ApiError> {
    match path.split("-").collect::<Vec<_>>().as_slice() {
        [_, checksum] => Ok(String::from(*checksum)),
        _ => Err(ApiError::NotFound),
    }
}
