mod admin;
mod api;
mod charge_price_api;
mod config;
mod db;
mod file_watcher;
mod fuchs_middleware;
mod image_import;
mod importer;
mod io;
mod log;
mod router;
mod slack;
mod state;
mod timer;

use std::net::SocketAddr;

use axum::extract::Extension;
use state::State;
use thiserror::Error;

use crate::{image_import::{CardFolder, CpoFolder, ImageFolder}, log::LogType};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = config::read_config().map_err(MainError::from)?;
    log::setup(LogType::Normal);

    tracing::info!("Creating database pool connection");

    let (timer, time_out) = timer::Timer::new(config.interval.to_std().expect("invalid interval"));

    let state = State::new(
        db::connect(&config.database_url, config.database_pool_size).await?,
        config.clone(),
        timer,
    );
    admin::init_admin_user(&state).await?;

    io::init_banner_folder().await?;

    if !config.replication {
        let card_folder = CardFolder::new();
        image_import::import_folder(&state, &card_folder).await?;
        file_watcher::watch_cards_folder(state.clone(), card_folder)?;

        let cpo_folder = CpoFolder::new();
        image_import::import_folder(&state, &cpo_folder).await?;
        file_watcher::watch_cards_folder(state.clone(), cpo_folder)?;

        importer::spawn_price_task(state.clone(), time_out);
        importer::spawn_cpo_task(state.clone())
    }

    let addr = SocketAddr::from((config.listen, config.port));
    tracing::info!("Listening on: {}", addr);

    let app = router::register(&config.admin_domain).layer(Extension(state));

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
