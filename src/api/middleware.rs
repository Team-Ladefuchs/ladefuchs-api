use axum::{
    http::{self, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

use crate::state::State;

use super::error::ApiError;

pub async fn auth<B>(req: Request<B>, next: Next<B>) -> Result<impl IntoResponse, ApiError> {
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
