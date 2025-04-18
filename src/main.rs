mod admin;
mod api;
mod charge_price_api;
mod config;
mod db;
mod eco_movement;
mod file_watcher;
mod image_import;
mod importer;
mod io;
mod log;
mod middleware;
mod router;
mod slack;
mod state;
mod timer;

use axum::extract::Extension;

use std::net::SocketAddr;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::{
    // image_import::{BannerFolder, CardFolder, ImageFolder, OperatorFolder},
    log::LogType,
};

use state::State;
use thiserror::Error;
use tokio::signal::unix::{SignalKind, signal};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = config::read_config().map_err(MainError::from)?;
    log::setup(LogType::Normal);

    tracing::info!("Creating database pool connection");

    let (timer, time_out) =
        timer::Timer::new(config.interval_minutes.to_std().expect("invalid interval"));

    let db_pool = db::connect(&config.database_url, config.database_pool_size).await?;
    let state = State::new(db_pool.clone(), config.clone(), timer);

    // admin::init_admin_user(&state).await?;
    // io::init_banner_folder().await?;

    if !config.replication {
        // images
        // let card_folder = CardFolder::new();
        // image_import::import_folder(&state, &card_folder).await?;
        // file_watcher::watch_image_folder(state.clone(), card_folder)?;

        // let operator_folder = OperatorFolder::new();
        // image_import::import_folder(&state, &operator_folder).await?;
        // file_watcher::watch_image_folder(state.clone(), operator_folder)?;

        // let banner_folder = BannerFolder::new();
        // image_import::import_folder(&state, &banner_folder).await?;
        // file_watcher::watch_image_folder(state.clone(), banner_folder)?;
        // images

        // background tasks
        // importer::spawn_price_task(state.clone(), time_out);
    }

    let _ = eco_movement::importer::import_data(state.clone()).await?;

    middleware::api_token_auth::spawn_token_task(state.clone());

    let app = router::register(&state.config)
        .layer(Extension(state))
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(log::set_span)
                .on_response(log::log_response)
                .on_request(log::log_request),
        );

    // exit on terminate or interrupt signal
    let mut term = signal(SignalKind::terminate()).unwrap();
    let mut int = signal(SignalKind::interrupt()).unwrap();

    tokio::task::spawn(async move {
        tokio::select! {
            _ = int.recv() => {}
            _ = term.recv() => {}
        }
        std::process::exit(0)
    });

    let addr = SocketAddr::from((config.listen, config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Ladefuchs version {}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Error, Debug)]
enum MainError {
    #[error(
        "environment configuration: `{}`. Please take a look at the README.md file, how to configure the server.", str::to_uppercase(&.0.to_string())
    )]
    Environment(#[from] envy::Error),
}
