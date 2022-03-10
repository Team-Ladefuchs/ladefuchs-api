use axum::{
    http::{self, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

use crate::state::State;

pub async fn auth<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let state: &State = req.extensions().get().unwrap();
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .map(|t| t.replace("Bearer ", ""));

    match auth_header {
        Some(token) if token.eq(&state.config.auth_token) => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
