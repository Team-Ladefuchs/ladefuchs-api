mod api;
mod charge_price_api;
mod config;
mod db;
mod log;
mod model;
mod state;
mod worker;

use axum::{extract::Extension, middleware};
use state::State;
use std::net::SocketAddr;
use thiserror::Error;

use tower_http::{compression::CompressionLayer, trace::TraceLayer};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = config::read_config().map_err(MainError::from)?;
    log::setup(config.log_type);

    tracing::info!("Starting Ladefuchs API 🦊");

    tracing::info!("Creating database pool connection");
    let state = State::new(db::connect(&config.database_url).await?, config.clone());

    // start import schedule
    worker::spawn_import_task(worker::hours(config.interval_h), state.clone());

    let addr = SocketAddr::from((config.listen, config.port));
    tracing::info!("Listening on: {}", addr);

    let app = api::route::register()
        .layer(middleware::from_fn(api::middleware::auth))
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
