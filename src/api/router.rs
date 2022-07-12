use super::{util::fmt_card_path, CardVersion};
use crate::api::endpoint;
use crate::api::endpoint::handler_404;
use crate::{admin, fuchs_middleware, log};
use axum::http::header::{
    ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
    ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE,
};
use axum::routing::post;
use axum::{handler::Handler, middleware, routing::get, Router};
use reqwest::Method;
use tower_cookies::CookieManagerLayer;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

use tower_http::trace::TraceLayer;
pub fn register(admin_domain: &url::Url) -> axum::Router {
    let domain = admin_domain.origin().unicode_serialization().to_string();
    let origins = [domain.parse().unwrap()];

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_credentials(true)
        .allow_headers([
            ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_METHODS,
            CONTENT_TYPE,
            ACCESS_CONTROL_ALLOW_ORIGIN,
            ACCESS_CONTROL_ALLOW_CREDENTIALS,
        ])
        .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS]);

    let admin = Router::new()
        .route("/login", post(admin::endpoints::login))
        .route("/logout", post(admin::endpoints::logout))
        .route("/confirm", get(admin::endpoints::verify_login))
        .route_layer(cors.clone());
    let admin_secure = Router::new()
        .route("/tariffs", get(admin::endpoints::get_all_tariffs))
        .route("/img/:checksum", get(endpoint::card_image))
        .route("/operators", get(admin::endpoints::get_all_cpos))
        .route_layer(cors)
        .route_layer(middleware::from_fn(fuchs_middleware::admin_auth));

    let api = Router::new()
        .route(fmt_card_path(CardVersion::V1), get(endpoint::cards_v1))
        .route(fmt_card_path(CardVersion::V2), get(endpoint::cards_v2))
        .route("/img/card/:file", get(endpoint::card_image))
        .route("/img/cards", get(endpoint::all_card_images))
        .route("/operators/:filter", get(endpoint::operators))
        .route("/v2/operators/:filter", get(endpoint::operators_v2))
        .route("/banners", get(endpoint::get_affiliate_banners))
        .route("/img/banner/:file", get(endpoint::get_banner_image))
        .route_layer(middleware::from_fn(fuchs_middleware::token_auth));

    let public = Router::new().route("/affiliate", get(endpoint::redirect_affiliate));

    Router::new()
        .nest("/admin", admin.nest("/auth", admin_secure))
        .nest("/", api)
        .nest("/", public)
        .layer(CookieManagerLayer::new())
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(log::set_span)
                .on_response(log::log_response)
                .on_request(log::log_request),
        )
        .fallback(handler_404.into_service())
}
