use crate::api::endpoint;
use axum::{routing::get, Router, handler::Handler};

use crate::api::endpoint::handler_404;

use super::{util::fmt_card_path, CardVersion};

pub fn register() -> axum::Router {
    Router::new()
        .route(fmt_card_path(CardVersion::V1), get(endpoint::cards_v1))
        .route(fmt_card_path(CardVersion::V2), get(endpoint::cards_v2))
        .route(fmt_card_path(CardVersion::V3), get(endpoint::cards_v3))
        .route("/operators/:filter", get(endpoint::operators))
        .fallback(handler_404.into_service())
}
