use axum::{
    extract::{Query, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{self, Request},
    middleware::Next,
    response::Response,
};

use tower_cookies::Cookies;

use crate::admin::endpoints::COOKIE_KEY;
use crate::{admin::endpoints::COOKIE_NAME, api::error::ApiError, state::State};

#[derive(Debug, serde::Deserialize)]
pub struct AuthParams {
    #[serde(rename = "apiKey")]
    api_key: Option<String>,
}

pub async fn token_auth<B>(
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    auth_query: Query<AuthParams>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    let state: &State = req.extensions().get().ok_or_else(|| ApiError::State)?;

    if matches!(auth_query.api_key.as_ref(), Some(key) if key == (&state.config.auth_token)) {
        return Ok(next.run(req).await);
    }

    match auth_header {
        Some(header) if header.token() == state.config.auth_token => Ok(next.run(req).await),
        Some(header) => Err(ApiError::WrongToken(header.token().to_string())),
        None => Err(ApiError::MissingToken),
    }
}

pub async fn admin_auth<B>(
    cookies: Cookies,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    if req.method() == http::Method::OPTIONS {
        return Ok(next.run(req).await);
    }
    match cookies.private(&COOKIE_KEY).get(COOKIE_NAME) {
        Some(_) => Ok(next.run(req).await),
        None => Err(ApiError::Login),
    }
}
