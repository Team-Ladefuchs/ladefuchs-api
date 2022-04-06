use crate::api::handler;
use axum::{routing::get, Router};

use super::{util::fmt_card_path, CardVersion};

pub fn register() -> axum::Router {
    Router::new()
        .route(fmt_card_path(CardVersion::V1), get(handler::cards_v1))
        .route(fmt_card_path(CardVersion::V2), get(handler::cards_v2))
        .route(fmt_card_path(CardVersion::V3), get(handler::cards_v3))
        .route("/operators/:filter", get(handler::operators))
}
