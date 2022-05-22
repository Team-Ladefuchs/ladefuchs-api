use crate::api::handler;
use axum::{handler::Handler, routing::get, Router};

use crate::api::handler::handler_404;

use super::{util::fmt_card_path, CardVersion};

pub fn register() -> axum::Router {
    Router::new()
        .route("/harrarrrr", get_service(ServeDir::new(".")))
        .route(fmt_card_path(CardVersion::V1), get(handler::cards_v1))
        .route(fmt_card_path(CardVersion::V2), get(handler::cards_v2))
        .route(fmt_card_path(CardVersion::V3), get(handler::cards_v3))
        .route("/operators/:filter", get(handler::operators))
        .fallback(handler_404.into_service())
}
