use axum::{body::Body, extract::Query, http::Request, middleware::Next, response::Response};

use axum_extra::TypedHeader;
use headers::{Authorization, authorization::Bearer};

use crate::{api::error::ApiError, ladefuchs_db::token::get_api_token, state::State};

#[derive(Debug, serde::Deserialize)]
pub struct AuthParams {
    #[serde(rename = "apiKey")]
    api_key: Option<String>,
}

pub async fn token_auth(
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    auth_query: Query<AuthParams>,
    req: Request<Body>,
    next: Next,
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
