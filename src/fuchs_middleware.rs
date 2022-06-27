use axum::{
    extract::{FromRequest, RequestParts},
    http::{self, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use cookie::Cookie;
use tower_cookies::Cookies;

use crate::admin::endpoints::COOKIE_KEY;
use crate::{admin::endpoints::COOKIE_NAME, api::error::ApiError, state::State};

pub async fn token_auth<B>(req: Request<B>, next: Next<B>) -> Result<impl IntoResponse, ApiError> {
    let state: &State = req.extensions().get().ok_or_else(|| ApiError::State)?;
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .map(|t| t.replace("Bearer ", ""));

    let ret = match auth_header {
        Some(token) if token.eq(&state.config.auth_token) => Ok(next.run(req).await),
        Some(token) => Err((StatusCode::UNAUTHORIZED, ApiError::WrongToken(token))),
        None => Err((StatusCode::UNAUTHORIZED, ApiError::MissingToken)),
    };
    Ok(ret)
}

pub async fn admin_auth<B: std::marker::Send>(
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, ApiError> {
    dbg!(&req.extensions());
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
