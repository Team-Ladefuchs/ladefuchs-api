use crate::admin::jwt_auth::{AdminAuthToken, ADMIN_COOKIE_NAME};
use crate::api::error::ApiError;

use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::CookieJar;

pub async fn admin_auth_token(
    jar: CookieJar,
    req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    match jar.get(ADMIN_COOKIE_NAME).map(|c| c.value()) {
        Some(token) if AdminAuthToken::is_valid(token) => Ok(next.run(req).await),
        _ => Err(ApiError::MissingToken),
    }
}
