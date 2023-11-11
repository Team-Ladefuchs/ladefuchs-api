use axum::{
    http::header::{
        ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
        ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE,
    },
    middleware,
    routing::{get, patch, post},
    Router,
};
use axum_login::login_required;
use reqwest::Method;

use tower_http::cors::CorsLayer;
use url::Url;

use crate::{
    admin::{self},
    api::endpoint,
    fuchs_middleware,
};

pub fn register(admin_domain: &Url) -> axum::Router {
    let cors = config_cors(admin_domain);

    let admin = admin_router(cors);

    let api = api_router();

    let public = Router::new().route("/", get(endpoint::affiliate::redirect_affiliate));

    Router::new()
        .nest("/admin", admin)
        .nest("/", api)
        .nest("/affiliate", public)
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
            "/cards/de/:cpo_name/:charge_type",
            get(endpoint::charge_conditions::v1::cards),
        )
        .route(
            "/v2/cards/de/:cpo_name/:charge_type",
            get(endpoint::charge_conditions::v2::cards),
        )
        .route(
            "/v3/conditions",
            post(endpoint::charge_conditions::v3::charge_conditions_with_filter),
        )
        .route(
            "/v3/conditions/:operator_id",
            get(endpoint::charge_conditions::v3::charge_conditions),
        )
        .route(
            "/v2/cards/de",
            post(endpoint::charge_conditions::v2::card_by_operators_and_tariffs),
        )
        .route("/operators/:filter", get(endpoint::operators::v1::get))
        .route("/v2/operators/:filter", get(endpoint::operators::v2::get))
        .route("/v3/operators", get(endpoint::operators::v3::get))
        .route("/banners", get(endpoint::images::get_affiliate_banners))
        .route("/v3/tariffs", get(endpoint::tariffs::v3::get_all))
        .route_layer(middleware::from_fn(fuchs_middleware::token_auth));

    api.merge(api_img)
}

fn admin_router(cors: CorsLayer) -> Router {
    let admin = Router::new()
        .route("/logout", post(admin::auth::logout))
        .route("/login", post(admin::auth::login))
        .route_layer(cors.clone());

    let admin_auth = Router::new()
        .route("/tariffs", get(admin::api_endpoints::get_all_tariffs))
        .route("/tariff", patch(admin::api_endpoints::patch_tariff))
        .route(
            "/stats/banner/:day/:link_id",
            get(admin::api_endpoints::get_banner_chart_data),
        )
        .route(
            "/stats/banner/summary/:link_id",
            get(admin::api_endpoints::get_banner_statistics),
        )
        .route("/img/card/:file", get(endpoint::images::img_by_checksum))
        .route("/operator", patch(admin::api_endpoints::patch_operator))
        .route(
            "/operators",
            get(admin::api_endpoints::get_all_standard_operators),
        )
        .route(
            "/operators/search",
            post(admin::api_endpoints::operator_search),
        )
        .route(
            "/import/start",
            post(admin::api_endpoints::trigger_manual_import),
        )
        .route("/confirm", get(admin::auth::confirm_login))
        .route("/import/last", get(admin::api_endpoints::last_import))
        .route_layer(login_required!(admin::auth::Backend))
        .route_layer(cors);

    admin.nest("/auth", admin_auth)
}

pub fn config_cors(admin_domain: &url::Url) -> CorsLayer {
    let domain = admin_domain.origin().unicode_serialization().to_string();
    let origins = [domain.parse().unwrap()];

    dbg!(&origins);
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
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
}
