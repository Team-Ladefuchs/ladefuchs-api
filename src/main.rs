mod api;
mod charge_price_api;
mod config;
mod db;
mod importer;
mod log;
mod state;

use axum::http::header::AUTHORIZATION;
use axum::{extract::Extension, middleware};
use chrono::Duration;
use reqwest::Method;
use state::State;
use std::{iter::once, net::SocketAddr};
use thiserror::Error;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer,
    sensitive_headers::SetSensitiveRequestHeadersLayer, trace::TraceLayer,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = config::read_config().map_err(MainError::from)?;
    log::setup(config.log_type);

    tracing::info!("Starting Ladefuchs API 🦊");

    tracing::info!("Creating database pool connection");
    let state = State::new(
        db::connect(&config.database_url, config.database_pool_size).await?,
        config.clone(),
    );

    // start import schedule
    importer::spawn_background_task(importer::hours(config.interval_h), state.clone());

    let addr = SocketAddr::from((config.listen, config.port));
    tracing::info!("Listening on: {}", addr);

    let cors = CorsLayer::new()
        .max_age(Duration::hours(1).to_std()?)
        .allow_credentials(true)
        .allow_origin(tower_http::cors::Any)
        .allow_methods(vec![Method::GET]);

    let app = api::router::register()
        .layer(cors)
        .layer(middleware::from_fn(api::middleware::auth))
        .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION)))
        .layer(Extension(state))
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(log::set_span)
                .on_response(log::log_response)
                .on_request(log::log_request),
        );

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Error, Debug)]
enum MainError {
    #[error(
        "environment configuration: `{}`. Please take a look at the README.md file, how to configure the server.", str::to_uppercase(&.0.to_string())
    )]
    Environment(#[from] envy::Error),
}
