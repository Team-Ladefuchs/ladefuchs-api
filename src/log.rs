use std::time::Duration;

use axum::{
    body::Body,
    http::{Request, Response},
};
use tracing::Span;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(serde::Deserialize, Clone, Debug, Copy)]
pub enum LogType {
    Normal,
    Json,
}

impl Default for LogType {
    fn default() -> Self {
        LogType::Normal
    }
}

pub fn setup(log_type: LogType) {
    let log_key = "LOG";
    if std::env::var_os(log_key).is_none() {
        std::env::set_var(log_key, "info");
    }
    let show_source = cfg!(debug_assertions);
    let builder = FmtSubscriber::builder()
        .pretty()
        .with_env_filter(EnvFilter::from_env(log_key))
        .with_line_number(show_source)
        .with_ansi(true)
        .with_target(show_source)
        .without_time()
        .with_file(show_source)
        .compact();

    match log_type {
        LogType::Normal => {
            builder.init();
        }
        LogType::Json => {
            builder.json().init();
        }
    };
}

pub fn log_response<B>(response: &Response<B>, latency: Duration, _span: &Span) {
    let status = response.status();
    tracing::info!(status = %status, latency=%format!("{}ms", latency.as_millis()))
}

pub fn set_span(_request: &Request<Body>) -> Span {
    tracing::info_span!(
        "http-request:",
        "user-agent" = tracing::field::Empty,
        method = tracing::field::Empty,
        path = tracing::field::Empty
    )
}

pub fn log_request(request: &Request<Body>, span: &Span) {
    if let Some(user_agent) = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
    {
        span.record("user-agent", &tracing::field::display(user_agent));
    }

    span.record("method", &tracing::field::display(request.method()));
    span.record("path", &tracing::field::display(request.uri().path()));

    // tracing::info!("started {} {}", request.method(), request.uri().path())
}
