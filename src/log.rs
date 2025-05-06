use std::time::Duration;

use axum::{
    body::Body,
    http::{Request, Response},
};
use tracing::Span;
use tracing_subscriber::FmtSubscriber;

#[derive(serde::Deserialize, Clone, Debug, Copy, Default)]
pub enum LogType {
    #[default]
    Normal,
    Json,
}

pub fn setup(log_type: LogType) {
    let show_source = cfg!(debug_assertions);
    let builder = FmtSubscriber::builder()
        .pretty()
        .with_line_number(show_source)
        .with_ansi(true)
        .with_target(show_source)
        .with_file(show_source)
        .compact();

    match (log_type, !cfg!(debug_assertions)) {
        (LogType::Normal, true) => builder.without_time().init(),
        (LogType::Json, true) => builder.json().init(),
        (LogType::Normal, _) => builder.init(),
        (LogType::Json, _) => builder.without_time().json().init(),
    };
}

pub fn log_response<B>(response: &Response<B>, latency: Duration, _span: &Span) {
    let status = response.status();
    tracing::debug!(status = %status, latency=%format!("{}ms", latency.as_millis()))
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
    span.record("method", tracing::field::display(request.method()));
    span.record("path", tracing::field::display(request.uri().path()));
}
