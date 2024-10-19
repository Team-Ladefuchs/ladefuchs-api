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
    let token = jar
        .get(ADMIN_COOKIE_NAME)
        .map(|cookie| cookie.value())
        .ok_or(ApiError::MissingToken)?;

    if !AdminAuthToken::is_valid(token) {
        return Err(ApiError::LoginTimeOut);
    }

    Ok(next.run(req).await)
}
