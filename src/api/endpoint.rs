use super::card;
use super::error::ApiError;
use super::operator::{self, Filter};
use super::util::{json, json_list};
use super::{ApiJsonList, RequestCardPath};
use crate::db::banner::PlattformType;
use crate::db::{self, banner, charge_price};
use crate::io;
use crate::io::FileStream;
use crate::state::State;
use axum::body::Body;
use axum::extract::rejection::PathRejection;
use axum::extract::{Extension, Path, Query};
use axum::http::{header, Request};
use axum::response::Redirect;
use serde::Deserialize;

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
    let operators = db::cpo::get_operators::<operator::Operator>(
        &mut state.database_pool.acquire().await?,
        filter,
    )
    .await?;

    json(operators)
}

pub async fn operators_v2(
    Extension(state): Extension<State>,
    path: Result<Path<Filter>, PathRejection>,
) -> ApiJsonList<operator::OperatorV2> {
    let Path(filter) = path?;
    let operators = db::cpo::get_operators::<operator::OperatorV2>(
        &mut state.database_pool.acquire().await?,
        filter,
    )
    .await?;
    json(operators)
}

pub async fn handler_404() -> ApiError {
    ApiError::NotFound
}

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
    dbg!(&file);
    let resp = io::read_file_stream(&file).await?;

    Ok(resp)
}

#[derive(Deserialize, Debug)]
pub struct AffilateParams {
    reference: url::Url,
}

pub async fn redirect_affiliate(
    Extension(state): Extension<State>,
    params: Query<AffilateParams>,
    req: Request<Body>,
) -> Result<Redirect, ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let link_id = banner::link_id(&mut connection, &params.reference).await;
    if link_id.is_none() {}
    match link_id {
        Some(id) => {
            let user_agent = &req
                .headers()
                .get("user-agent")
                .map(|header| header.to_str().unwrap_or_default())
                .map(|agent| PlattformType::from(agent));
            if let Some(plattform) = user_agent {
                banner::update_link_states(&mut connection, id, plattform)
                    .await
                    .ok();
            }
        }
        None => return Err(ApiError::BadRequest),
    }

    Ok(Redirect::permanent(params.reference.as_str()))
}
