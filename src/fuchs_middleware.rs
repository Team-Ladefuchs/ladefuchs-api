use axum::{
    extract::{Query, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{self, Request},
    middleware::Next,
    response::Response,
};

use sqlx::{pool::PoolConnection, Postgres};
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

    let mut connection = state.database_pool.acquire().await?;

    if matches!(auth_query.api_key.as_ref(), Some(token) if check_api_token(&mut connection, token).await)
    {
        return Ok(next.run(req).await);
    }

    match auth_header {
        Some(header) if check_api_token(&mut connection, header.token()).await => {
            Ok(next.run(req).await)
        }
        Some(header) => Err(ApiError::WrongToken(header.token().to_string())),
        None => Err(ApiError::MissingToken),
    }
}

pub async fn check_api_token(connection: &mut PoolConnection<Postgres>, token: &str) -> bool {
    let result = sqlx::query_file_scalar!("sql/get/check_token.sql", token)
        .fetch_optional(connection)
        .await;

    match result {
        Ok(value) if value.is_some() => true,
        _ => false,
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
