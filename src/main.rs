mod api;
mod charge_price_api;
mod config;
mod db;
mod log;
mod model;
mod state;
mod worker;

use axum::{handler::Handler, middleware, AddExtensionLayer, BoxError};
use state::State;
use std::net::SocketAddr;
use thiserror::Error;

use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::api::handler::handler_404;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let config = config::read_config()?;

    log::setup(config.log_type);
    let state = State::new(db::connect(&config.database_url).await?, config.clone());

    worker::spawn_import_task(worker::hours(config.interval_h), state.clone());

    let app = api::route::register()
        .layer(middleware::from_fn(api::middleware::auth))
        .layer(AddExtensionLayer::new(state))
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(log::set_span)
                .on_response(log::log_response)
                .on_request(log::log_request),
        )
        .fallback(handler_404.into_service());

    let addr = SocketAddr::from((config.listen, config.port));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(MainError::map)?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum MainError {
    #[error(
        "Config environment error: `{0}`. Please take a look at the README.md file, how to configure the server."
    )]
    Disconnect(#[from] envy::Error),
    #[error("Server: {0}")]
    Sever(#[from] axum::Error),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl MainError {
    fn map(error: impl Into<BoxError>) -> Self {
        MainError::Sever(axum::Error::new(error))
    }
}
