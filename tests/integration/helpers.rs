use std::path::PathBuf;

use axum::{Extension, Router, body::to_bytes, http::Uri};
use reqwest::header::AUTHORIZATION;
use sqlx::PgPool;
use tower::ServiceExt;

use ladefuchs_api::{admin, config, router, state::State};

pub struct TestClient {
    authorized: bool,
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

#[derive(Debug)]
pub struct TestResponse(axum::response::Response);

impl TestClient {
    pub async fn new(pool: PgPool) -> Self {
        let config = config();

        let state = State::new(pool, config.clone());
        state.tokens.write().await.insert("111".to_owned());

        admin::init_admin_user(&state)
            .await
            .expect("could not init admin user");

        let app = router::register(&state.config).layer(Extension(state));

        Self {
            router: app,
            authorized: false,
        }
    }

    pub fn authorized(mut self) -> Self {
        self.authorized = true;
        self
    }

    pub async fn get<T>(&self, uri: T) -> TestResponse
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: Into<axum::http::Error>,
    {
        let mut request = axum::http::Request::get(uri)
            .body(axum::body::Body::empty())
            .expect("could not create request");

        if self.authorized {
            request
                .headers_mut()
                .insert(AUTHORIZATION, "Bearer 111".parse().unwrap());
        }

        TestResponse(
            self.router
                .clone()
                .oneshot(request)
                .await
                .expect("request failed"),
        )
    }

    pub async fn get_with_user_agent<T>(&self, uri: T, user_agent: impl AsRef<str>) -> TestResponse
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: Into<axum::http::Error>,
    {
        let mut request = axum::http::Request::get(uri)
            .header("user-agent", user_agent.as_ref())
            .body(axum::body::Body::empty())
            .expect("could not create request");

        if self.authorized {
            request
                .headers_mut()
                .insert(AUTHORIZATION, "Bearer 111".parse().unwrap());
        }

        TestResponse(
            self.router
                .clone()
                .oneshot(request)
                .await
                .expect("request failed"),
        )
    }
}

impl TestResponse {
    pub fn status(&self) -> axum::http::StatusCode {
        self.0.status()
    }

    pub async fn json<T>(self) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        let body = self.0.into_body();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[allow(dead_code)]
    pub async fn text(self) -> String {
        let body = self.0.into_body();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    pub fn headers(&self) -> &axum::http::HeaderMap {
        self.0.headers()
    }
}
