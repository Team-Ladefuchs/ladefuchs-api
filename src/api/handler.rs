use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use axum::Json;
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
};

use reqwest::StatusCode;

use crate::db::charging::ChargeType;
use crate::db::price::{self, ChargePrice};
use crate::state::State;

pub async fn hello(
    Extension(state): Extension<State>,
    // Path(cpo_name): Path<String>,
    Path((cpo_name, charge_type)): Path<(String, ChargeType)>,
) -> Json<Vec<ChargePrice>> {
    // tracing::debug!(c = ?charge_type);
    let prices = price::get(charge_type, &cpo_name, &state.database_pool)
        .await
        .unwrap();
    axum::Json(prices)
}

pub async fn auth(
    Extension(state): Extension<State>,
    ExtractToken(token): ExtractToken,
) -> &'static str {
    dbg!(state.config.auth_token.clone());
    if token.eq(&state.config.auth_token) {
        "Hello, auth!"
    } else {
        "No auth2"
    }
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Resource not found")
}

// pub async fn check_auth<B>(
//     response: &Response<B>,
//     ExtractUserAgent(token): ExtractUserAgent,
//     ,
// ) -> impl IntoResponse {

// }

pub struct ExtractToken(String);

#[async_trait]
impl<B> FromRequest<B> for ExtractToken
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        dbg!(req.headers());
        req.headers()
            .and_then(|headers| headers.get("authorization"))
            .and_then(|value| value.to_str().ok())
            .map(|t| t.replace("Bearer ", ""))
            .map(|token| ExtractToken(token))
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    "`authorization token` header is missing",
                )
            })
    }
}
