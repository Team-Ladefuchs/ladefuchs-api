use axum::{
    http::header::{
        ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
        ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE,
    },
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use reqwest::Method;
use tower_cookies::CookieManagerLayer;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use crate::{
    admin,
    api::{endpoint, util::fmt_card_path, CardVersion},
    fuchs_middleware, log,
};

pub fn register(admin_domain: &url::Url) -> axum::Router {
    let cors = config_cors(admin_domain);

    let admin = admin_router(cors);

    let api = api_router();

    let public = Router::new().route("/", get(endpoint::affiliate::redirect_affiliate));

    Router::new()
        .nest("/admin", admin)
        .nest("/", api)
        .nest("/affiliate", public)
        .layer(CookieManagerLayer::new())
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(log::set_span)
                .on_response(log::log_response)
                .on_request(log::log_request),
        )
        .fallback(endpoint::handler_404)
}

fn api_router() -> Router {
    let api_img = Router::new()
        .route(
            "/img/card/:file_checksum",
            get(endpoint::images::img_by_checksum),
        )
        .route(
            "/img/cpo/:file_checksum",
            get(endpoint::images::img_by_checksum),
        )
        .route(
            "/img/banner/:file_checksum",
            get(endpoint::images::img_by_checksum),
        )
        .route("/img/cards", get(endpoint::images::all_card_images))
        .route("/img/cpos", get(endpoint::images::all_cpo_images));

    let api = Router::new()
        .route(
            fmt_card_path(CardVersion::V1),
            get(endpoint::cards::cards_v1),
        )
        .route(
            fmt_card_path(CardVersion::V2),
            get(endpoint::cards::cards_v2),
        )
        .route("/operators/:filter", get(endpoint::operators::get))
        .route("/v2/operators/:filter", get(endpoint::operators::get_v2))
        .route("/msps", get(endpoint::msps::get_all))
        .route("/banners", get(endpoint::images::get_affiliate_banners))
        .route_layer(middleware::from_fn(fuchs_middleware::token_auth));

    api.merge(api_img)
}

fn admin_router(cors: CorsLayer) -> Router {
    let admin = Router::new()
        .route("/logout", post(admin::endpoints::logout))
        .route("/login", post(admin::endpoints::login))
        .route_layer(cors.clone());

    let admin_auth = Router::new()
        .route("/tariffs", get(admin::endpoints::get_all_tariffs))
        .route(
            "/stats/banner/:day/:link_id",
            get(admin::endpoints::get_banner_chart_data),
        )
        .route(
            "/stats/banner/summary/:link_id",
            get(admin::endpoints::get_banner_statistics),
        )
        .route("/img/card/:file", get(endpoint::images::img_by_checksum))
        .route("/operator/:cpo_id", delete(admin::endpoints::delete_cpo))
        .route("/operator", put(admin::endpoints::insert_update_cpo))
        .route("/operators", get(admin::endpoints::get_all_cpos))
        .route("/operators/search", post(admin::endpoints::cpo_search))
        .route(
            "/import/start",
            post(admin::endpoints::trigger_manual_import),
        )
        .route("/confirm", get(admin::endpoints::confirm_login))
        .route("/import/last", get(admin::endpoints::last_import))
        .route_layer(cors)
        .route_layer(middleware::from_fn(fuchs_middleware::admin_auth));

    admin.nest("/auth", admin_auth)
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
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
}
