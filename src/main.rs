mod admin;
mod api;
mod charge_price_api;
mod config;
mod db;
mod fuchs_middleware;
mod importer;
mod io;
mod log;
mod router;
mod slack;
mod state;
mod tariff_image;

use axum::extract::Extension;
use state::State;
use std::net::SocketAddr;
use thiserror::Error;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = config::read_config().map_err(MainError::from)?;
    log::setup(config.log_type);

    tracing::info!("Creating database pool connection");
    let state = State::new(
        db::connect(&config.database_url, config.database_pool_size).await?,
        config.clone(),
    );
    admin::init_admin_user(&state).await?;

    io::init_banner_folder().await?;

    if !config.replication {
        tariff_image::import_folder(&state).await?;
        tariff_image::watch_folder(state.clone())?;

        importer::spawn_background_task(importer::hours(config.interval), state.clone());
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
