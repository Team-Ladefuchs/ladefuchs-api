use axum::{
    http::{
        header::{
            ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE,
        },
        Method,
    },
    middleware,
    routing::{get, patch, post},
    Router,
};
use axum_login::login_required;

use tower_http::cors::CorsLayer;
use url::Url;

use crate::{
    admin,
    api::{
        self, affiliate, banner, charge_condition, charge_price_ad, feedback, image, operator,
        tariff,
    },
    fuchs_middleware,
};

pub fn register(admin_domain: &Url) -> axum::Router {
    let cors = config_cors(admin_domain);

    let admin = admin_router(cors);

    let api = api_router();

    let public = Router::new().route("/", get(affiliate::redirect_affiliate));

    Router::new()
        .nest("/admin", admin)
        .nest("/", api)
        .nest("/affiliate", public)
        .fallback(api::handler_404)
}

fn api_router() -> Router {
    let images = Router::new()
        .route("/image/:file_checksum", get(image::image_by_checksum))
        .route("/img/card/:file_checksum", get(image::image_by_checksum))
        .route("/img/cpo/:file_checksum", get(image::image_by_checksum))
        .route("/img/banner/:file_checksum", get(image::image_by_checksum))
        .route("/image/proxy", get(image::image_proxy));

    let api_v1 = Router::new()
        .route(
            "/cards/de/:cpo_name/:charge_type",
            get(charge_condition::v1::get_handler),
        )
        .route("/operators/:filter", get(operator::v1::get_handler));

    let api_v2 = Router::new()
        .route("/v2/operators/:filter", get(operator::v2::get_handler))
        .route("/banners", get(banner::v2::get_handler))
        .route(
            "/v2/cards/de/:cpo_name/:charge_type",
            get(charge_condition::v2::get_handler),
        )
        .route("/v2/cards/de", post(charge_condition::v2::post_handler));

    let api_v3 = Router::new()
        .route("/v3/conditions", post(charge_condition::v3::post_handler))
        .route(
            "/v3/conditions/:operator_id",
            get(charge_condition::v3::get_handler),
        )
        .route("/v3/operators", get(operator::v3::get_handler))
        .route("/v3/tariffs", get(tariff::v3::get_handler))
        .route("/v3/banners", get(banner::v3::get_handler))
        .route(
            "/v3/banners/chargeprice/advertisement",
            get(charge_price_ad::v3::get_handler),
        )
        .route("/v3/images", get(image::v3::get_handler))
        .route("/v3/feedback", post(feedback::v3::post_handler));

    let api = Router::new()
        .merge(images)
        .merge(api_v1)
        .merge(api_v2)
        .merge(api_v3)
        .route_layer(middleware::from_fn(fuchs_middleware::token_auth));

    api
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
        .route("/img/card/:file", get(image::image_by_checksum))
        .route("/operator", patch(admin::api_endpoints::patch_operator))
        .route("/operators", get(admin::api_endpoints::get_operators))
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
        .route_layer(login_required!(admin::auth::Backend, login_url = "/login"))
        .route_layer(cors);

    admin.nest("/auth", admin_auth)
}

pub fn config_cors(admin_domain: &url::Url) -> CorsLayer {
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
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
}
