use crate::admin::jwt_auth::{AdminUser, ADMIN_COOKIE_NAME, JWT_KEYS};
use crate::api::error::ApiError;

use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, Validation};

pub async fn admin_auth_token(
    jar: CookieJar,
    req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    match jar.get(ADMIN_COOKIE_NAME).map(|c| c.value()) {
        Some(token)
            if decode::<AdminUser>(&token, &JWT_KEYS.decoding, &Validation::default())
                .map_err(|_| ApiError::LoginTimeOut)
                .ok()
                .is_some() =>
        {
            Ok(next.run(req).await)
        }
        _ => Err(ApiError::MissingToken),
    }
}
