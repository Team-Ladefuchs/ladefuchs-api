use axum::{
    body::Body,
    extract::{rejection::QueryRejection, Query},
    http::Request,
    response::Redirect,
    Extension,
};
use serde::Deserialize;

use crate::{
    db::banner::{self, PlattformType},
    state::State,
};

use super::error::ApiError;

pub mod cards;
pub mod images;
pub mod operators;

pub async fn handler_404() -> ApiError {
    ApiError::NotFound
}

#[derive(Deserialize, Debug)]
pub struct AffilateParams {
    url: url::Url,
}

pub async fn redirect_affiliate(
    Extension(state): Extension<State>,
    params: Result<Query<AffilateParams>, QueryRejection>,
    req: Request<Body>,
) -> Result<Redirect, ApiError> {
    let url = &params?.url;
    let mut connection = state.database_pool.acquire().await?;
    let link_id = banner::link_id(&mut connection, url).await;
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

    Ok(Redirect::permanent(url.as_str()))
}
