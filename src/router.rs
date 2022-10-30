use crate::api::{endpoint, CardVersion};

use crate::api::util::{banner_img_path, fmt_card_path};
use crate::{admin, fuchs_middleware, log};
use axum::handler::Handler;
use axum::http::header::{
    ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
    ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE,
};
use axum::{middleware, routing::get, routing::post, Router};
use reqwest::Method;
use tower_cookies::CookieManagerLayer;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

use tower_http::trace::TraceLayer;

pub fn register(admin_domain: &url::Url) -> axum::Router {
    let cors = config_cors(admin_domain);

    let admin = Router::new()
        .route("/login", post(admin::endpoints::login))
        .route("/logout", post(admin::endpoints::logout))
        .route("/confirm", get(admin::endpoints::verify_login))
        .route_layer(cors.clone());

    let admin_auth = Router::new()
        .route("/tariffs", get(admin::endpoints::get_all_tariffs))
        .route("/img/:filename", get(endpoint::images::card_image_by_name))
        .route(
            "/stats/banner/:day",
            get(admin::endpoints::get_banner_chart_data),
        )
        .route(
            "/stats/banner/summary",
            get(admin::endpoints::get_banner_statistics),
        )
        .route("/operators", get(admin::endpoints::get_all_cpos))
        .route("/operators/search", post(admin::endpoints::cpo_search))
        .route("/import/start", post(admin::endpoints::trigger_import))
        .route("/import/last", get(admin::endpoints::last_import))
        .route_layer(cors)
        .route_layer(middleware::from_fn(fuchs_middleware::admin_auth));

    let api = Router::new()
        .route(
            fmt_card_path(CardVersion::V1),
            get(endpoint::cards::cards_v1),
        )
        .route(
            fmt_card_path(CardVersion::V2),
            get(endpoint::cards::cards_v2),
        )
        .route("/img/card/:file", get(endpoint::images::card_image))
        .route("/img/cards", get(endpoint::images::all_card_images))
        .route("/operators/:filter", get(endpoint::operators::get))
        .route("/v2/operators/:filter", get(endpoint::operators::get_v2))
        .route("/msps", get(endpoint::msps::get_all))
        .route("/banners", get(endpoint::images::get_affiliate_banners))
        .route(banner_img_path(), get(endpoint::images::get_banner_image))
        .route_layer(middleware::from_fn(fuchs_middleware::token_auth));

    let public = Router::new().route("/affiliate", get(endpoint::affiliate::redirect_affiliate));

    Router::new()
        .nest("/admin", admin.nest("/auth", admin_auth))
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
        .fallback(endpoint::handler_404.into_service())
}

fn config_cors(admin_domain: &url::Url) -> CorsLayer {
    let domain = admin_domain.origin().unicode_serialization().to_string();
    let origins = [domain.parse().unwrap()];

    CorsLayer::new()
        .allow_origin(origins)
        .allow_credentials(true)
        .allow_headers([
            ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_METHODS,
            CONTENT_TYPE,
            ACCESS_CONTROL_ALLOW_ORIGIN,
            ACCESS_CONTROL_ALLOW_CREDENTIALS,
        ])
        .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
}
