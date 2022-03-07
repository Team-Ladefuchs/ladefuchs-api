mod api;
mod charge_price_api;
mod config;
mod db;
mod log;
mod model;
mod state;
mod worker;

use axum::{body::Body, http::Request, routing::get, Router};
use chrono::Duration;
use state::State;
use std::{net::SocketAddr, process};
use thiserror::Error;

use tower_http::{compression::CompressionLayer, trace::TraceLayer};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        tracing::error!("{:#}", e);
        process::exit(1);
    }
}

// #[instrument]
async fn run() -> Result<(), eyre::Error> {
    let config = config::read_config()?;

    log::setup(config.log_type);
    let state = State::new(db::connect(&config.database_url).await?, config.clone());

    worker::spaw_import_task(Duration::hours(i64::from(config.interval_h)), state).await?;

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(api::handler::hello))
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|_request: &Request<Body>| {
                    tracing::info_span!(
                        "http-request:",
                        "user-agent" = tracing::field::Empty,
                        method = tracing::field::Empty,
                        path = tracing::field::Empty
                    )
                })
                .on_response(log::log_response)
                .on_request(log::log_request),
        );
    let addr = SocketAddr::from((config.address, config.port));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|err| MainError::Sever(axum::Error::new(err)))?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum MainError {
    #[error(
        "Config enviroment error: `{0}`. Please take a look at the README.md file, how to configure the server."
    )]
    Disconnect(#[from] envy::Error),
    #[error("Server: {0}")]
    Sever(#[from] axum::Error),
}
