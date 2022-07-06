use std::collections::HashMap;

use axum::{
    extract::{FromRequest, RequestParts},
    http::{self, Request},
    middleware::Next,
    response::IntoResponse,
};
use tower_cookies::Cookies;

use crate::admin::endpoints::COOKIE_KEY;
use crate::{admin::endpoints::COOKIE_NAME, api::error::ApiError, state::State};

pub async fn token_auth<B>(req: Request<B>, next: Next<B>) -> Result<impl IntoResponse, ApiError> {
    let state: &State = req.extensions().get().ok_or_else(|| ApiError::State)?;
    let params = req.uri().query().map(|param| {
        param
            .split("&")
            .filter_map(|key| key.split_once("="))
            .collect::<HashMap<&str, &str>>()
    });
    if matches!(params.as_ref().and_then(|map| map.get("authKey")), Some(key) if key.eq(&state.config.auth_token))
    {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .map(|t| t.replace("Bearer ", ""));

    match auth_header {
        Some(token) if token.eq(&state.config.auth_token) => Ok(next.run(req).await),
        Some(token) => Err(ApiError::WrongToken(token)),
        None => Err(ApiError::MissingToken),
    }
}

pub async fn admin_auth<B: std::marker::Send>(
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, ApiError> {
    let mut parts = RequestParts::new(req);
    let cookies = Cookies::from_request(&mut parts)
        .await
        .ok()
        .and_then(|cookie| cookie.private(&COOKIE_KEY).get(COOKIE_NAME));

    match cookies {
        Some(_) => Ok(next.run(parts.try_into_request().unwrap()).await),
        None => Err(ApiError::Login),
    }
}
