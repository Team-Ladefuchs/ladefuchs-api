use std::path::PathBuf;

use axum::{Extension, Router, http::Uri};
use sqlx::PgPool;
use tower::ServiceExt;

use ladefuchs_api::{admin, config, router, state::State};

pub struct TestClient {
    router: Router,
}

fn config() -> config::Config {
    config::Config {
        database_url: "postgres://localhost/ladefuchs_test".parse().unwrap(),
        database_pool_size: 5,
        eco_movement_api_key: "".to_owned(),
        eco_movement_api_url: "https://example.com/".parse().unwrap(),
        port: 3000,
        listen: [127, 0, 0, 1].into(),
        cron_schedule: "0 45 23 * * *".to_owned(),
        domain: "http://127.0.0.1:3000".parse().unwrap(),
        slack_channel: None,
        slack_token: None,
        admin_user: None,
        admin_pwd: None,
        admin_domain: "http://127.0.0.1:8080".parse().unwrap(),
        docs_dir: PathBuf::from("./docs"),
        import_on_start: false,
        max_request_pages: 1000,
    }
}

impl TestClient {
    pub async fn new(pool: PgPool) -> Self {
        let config = config();

        let state = State::new(pool, config.clone());

        admin::init_admin_user(&state)
            .await
            .expect("could not init admin user");

        let app = router::register(&state.config).layer(Extension(state));

        Self { router: app }
    }

    pub async fn get<T>(&self, uri: T) -> axum::response::Response
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: Into<axum::http::Error>,
    {
        let request = axum::http::Request::get(uri)
            .body(axum::body::Body::empty())
            .expect("could not create request");

        self.router
            .clone()
            .oneshot(request)
            .await
            .expect("request failed")
    }
}
