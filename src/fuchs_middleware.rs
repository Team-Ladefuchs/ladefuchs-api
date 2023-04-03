use std::collections::HashSet;

use axum::{
    extract::{Query, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{self, Request},
    middleware::Next,
    response::Response,
};

use sqlx::PgPool;
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

    let header_token = auth_header
        .map(|auth| auth.token().to_owned())
        .or(auth_query.api_key.to_owned());

    match header_token {
        Some(token) if state.tokens.read().await.contains(&token) => Ok(next.run(req).await),
        Some(token) => Err(ApiError::WrongToken(token)),
        None => Err(ApiError::MissingToken),
    }
}

const fn thirty_minutes_duration() -> std::time::Duration {
    std::time::Duration::from_secs(60 * 30)
}

pub fn spawn_token_task(state: State) {
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(thirty_minutes_duration());
        loop {
            {
                match get_api_token(&state.database_pool).await {
                    Ok(tokens) => {
                        *state.tokens.write().await = tokens;
                        tracing::debug!(status = "token replaced");
                    }
                    Err(_) => tracing::debug!(status = "could not update tokens"),
                }
            }

            interval.tick().await;
        }
    });
}

pub async fn get_api_token(database_pool: &PgPool) -> Result<HashSet<String>, sqlx::Error> {
    let mut connection = database_pool.acquire().await?;
    let results = sqlx::query_file!("sql/get/tokens.sql")
        .fetch_all(&mut connection)
        .await?;
    Ok(results.into_iter().map(|row| row.value).collect())
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
