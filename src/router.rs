use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{
        Method,
        header::{
            ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE,
        },
    },
    middleware,
    routing::{get, patch, post},
};

use tower_http::{cors::CorsLayer, services::ServeDir};

const MAX_BODY_LIMIT: usize = 1024 * 1024 * 4; // 4MB

use crate::{
    admin::{self},
    api::{
        self, affiliate, announcement, app_metrics, banner, charge_condition, cp_legacy_ads,
        dynamic_price, feedback, image, operator, tariff,
    },
    config::Config,
    middleware::admin_token_auth::admin_auth_token,
};

pub fn register(config: &Config) -> axum::Router {
    let cors = config_cors(&config.admin_domain);

    let admin: Router = admin_router(cors);

    let api = api_router();

    let public = Router::new().route("/", get(affiliate::redirect_affiliate));

    Router::new()
        .layer(DefaultBodyLimit::max(MAX_BODY_LIMIT))
        .nest("/admin", admin)
        .merge(api)
        .nest("/affiliate", public)
        .fallback(api::handler_404)
}

fn api_router() -> Router {
    let images = Router::new()
        .route("/image/{file_checksum}", get(image::image_by_checksum))
        .route("/img/card/{file_checksum}", get(image::image_by_checksum))
        .route("/img/cpo/{file_checksum}", get(image::image_by_checksum))
        .route("/img/banner/{file_checksum}", get(image::image_by_checksum))
        .nest_service("/images/cards", ServeDir::new("./images/legacy_cards"))
        .route("/image/proxy", get(image::image_proxy));

    let api_v1 = Router::new()
        .route(
            "/cards/de/{cpo_name}/{charge_type}",
            get(charge_condition::v1::get_handler),
        )
        .route("/operators/{filter}", get(operator::v1::get_handler));

    let api_v2 = Router::new()
        .route("/v2/operators/{filter}", get(operator::v2::get_handler))
        .route("/banners", get(banner::v2::get_handler))
        .route(
            "/v2/cards/de/{cpo_name}/{charge_type}",
            get(charge_condition::v2::get_handler),
        )
        .route(
            "/v2/cards/de",
            post(charge_condition::v2::post_handler).get(api::handler_404),
        );

    let api_v3 = Router::new()
        .route("/v3/conditions", post(charge_condition::v3::post_handler))
        .route(
            "/v3/conditions/{operator_id}",
            get(charge_condition::v3::get_handler),
        )
        .route("/v3/operators", get(operator::v3::get_handler))
        .route("/v3/operators", post(operator::v3::post_handler))
        .route("/v3/tariffs", get(tariff::v3::get_handler))
        .route("/v3/tariffs", post(tariff::v3::post_handler))
        .route("/v3/dynamic-prices", post(dynamic_price::v3::post_handler))
        .route("/v3/banners", get(banner::v3::get_handler))
        .route("/v3/announcement", get(announcement::v3::get_handler))
        .route(
            "/v3/banners/impression",
            post(banner::v3::post_impression_handler),
        )
        .route(
            "/v3/banners/chargeprice/advertisement",
            get(cp_legacy_ads::v3::get_handler),
        )
        .route("/v3/app/metrics", post(app_metrics::v3::post_handler))
        .route("/v3/images", get(image::v3::get_handler))
        .route("/v3/feedback", post(feedback::v3::post_handler));

    let api_v4 = Router::new()
        .route("/v4/tariffs", get(tariff::v4::get_handler))
        .route("/v4/tariffs", post(tariff::v4::post_handler));

    Router::new()
        .merge(images)
        .merge(api_v1)
        .merge(api_v2)
        .merge(api_v3)
        .merge(api_v4)
        .route_layer(middleware::from_fn(
            crate::middleware::api_token_auth::token_auth,
        ))
}

fn admin_router(cors: CorsLayer) -> Router {
    let admin_auth = Router::new()
        .route("/tariff", patch(admin::api_endpoints::patch_tariff))
        .route("/image", post(admin::api_endpoints::post_image))
        .route("/img/{file}", get(image::image_by_checksum))
        .route("/operator", patch(admin::api_endpoints::patch_operator))
        .route_layer(middleware::from_fn(admin_auth_token))
        .route_layer(cors);

    Router::new().nest("/auth", admin_auth)
}

pub fn config_cors(admin_domain: &url::Url) -> CorsLayer {
    let domain = admin_domain.origin().unicode_serialization().to_string();

    CorsLayer::new()
        .allow_origin([domain.parse().unwrap()])
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
