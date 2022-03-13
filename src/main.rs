mod api;
mod charge_price_api;
mod config;
mod db;
mod log;
mod model;
mod state;
mod worker;

use axum::{body::Body, handler::Handler, http::Request, middleware, AddExtensionLayer};
use state::State;
use std::{net::SocketAddr, process};
use thiserror::Error;

use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::api::handler::handler_404;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        tracing::error!("{:#}", e);
        process::exit(1);
    }
}

async fn run() -> Result<(), eyre::Error> {
    let config = config::read_config()?;

    log::setup(config.log_type);
    let state = State::new(db::connect(&config.database_url).await?, config.clone());

    // worker::spawn_import_task(worker::hours(config.interval_h), state.clone());

    let app = api::route::register()
        .layer(middleware::from_fn(api::middleware::auth))
        .layer(AddExtensionLayer::new(state))
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
        )
        .fallback(handler_404.into_service());
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
