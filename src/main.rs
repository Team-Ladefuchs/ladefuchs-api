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

use axum::{error_handling::HandleErrorLayer, extract::Extension, BoxError};

use axum_login::AuthManagerLayer;
use reqwest::StatusCode;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tower_sessions::{cookie::SameSite, Expiry, PostgresStore, SessionManagerLayer};

use crate::{
    image_import::{BannerFolder, CardFolder, ImageFolder, OperatorFolder},
    log::LogType,
};

use state::State;
use thiserror::Error;
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = config::read_config().map_err(MainError::from)?;
    log::setup(LogType::Normal);

    tracing::info!("Creating database pool connection");

    let (timer, time_out) = timer::Timer::new(config.interval.to_std().expect("invalid interval"));

    let db_pool = db::connect(&config.database_url, config.database_pool_size).await?;
    let state = State::new(db_pool.clone(), config.clone(), timer);

    admin::init_admin_user(&state).await?;

    io::init_banner_folder().await?;

    if !config.replication {
        // images
        let card_folder = CardFolder::new();
        image_import::import_folder(&state, &card_folder).await?;
        file_watcher::watch_cards_folder(state.clone(), card_folder)?;

        let operator_folder = OperatorFolder::new();
        image_import::import_folder(&state, &operator_folder).await?;
        file_watcher::watch_cards_folder(state.clone(), operator_folder)?;

        let banner_folder = BannerFolder::new();
        image_import::import_folder(&state, &banner_folder).await?;
        file_watcher::watch_cards_folder(state.clone(), banner_folder)?;
        // images

        // background tasks
        importer::spawn_price_task(state.clone(), time_out);
        importer::spawn_operator_task(state.clone());
    }

    fuchs_middleware::spawn_token_task(state.clone());

    let session_store = PostgresStore::new(state.database_pool.clone());
    session_store.migrate().await.unwrap();

    let admin_backend = admin::auth::Backend::new(state.database_pool.clone());

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_path("/".to_string())
        .with_name("auth")
        .with_same_site(SameSite::Lax)
        .with_domain(
            state
                .as_ref()
                .config
                .admin_domain
                .host_str()
                .map(|host| host.replace("admin.", ""))
                .unwrap_or_default(),
        )
        .with_expiry(Expiry::OnInactivity(time::Duration::days(12)));

    let auth_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(AuthManagerLayer::new(admin_backend, session_layer));

    let app = router::register(&state.config.admin_domain)
        .layer(Extension(state))
        .layer(auth_service)
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
    tracing::info!("Ladefuchs version {}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Listening on http://{}", addr);
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
