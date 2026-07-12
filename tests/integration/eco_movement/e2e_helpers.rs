use ladefuchs_api::{config, state::State};
use sqlx::PgPool;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path, query_param},
};

use crate::helpers;

pub fn config_with_mock(mock_uri: &str) -> config::Config {
    let mut cfg = helpers::config();
    cfg.eco_movement_api_url = mock_uri.parse().expect("mock uri must be a valid url");
    cfg.max_request_pages = 1;
    cfg.import_on_start = false;
    cfg
}

pub fn build_state(pool: PgPool, mock_uri: &str) -> State {
    State::new(pool, config_with_mock(mock_uri))
}

const EMPTY_PAGE: &str = r#"{"data":[]}"#;

pub async fn mount_eco_endpoints(
    mock: &MockServer,
    locations_json: &'static str,
    prices_json: &'static str,
    connector_prices_json: &'static str,
) {
    mount_endpoint(mock, "/api/ocpi/cpo/2.1.1/locations", locations_json).await;
    mount_endpoint(mock, "/prices", prices_json).await;
    mount_endpoint(mock, "/prices/connector_prices", connector_prices_json).await;
}

async fn mount_endpoint(mock: &MockServer, p: &'static str, first_page: &'static str) {
    Mock::given(method("GET"))
        .and(path(p))
        .and(query_param("offset", "0"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_string(first_page),
        )
        .mount(mock)
        .await;

    Mock::given(method("GET"))
        .and(path(p))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_string(EMPTY_PAGE),
        )
        .mount(mock)
        .await;
}
