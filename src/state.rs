use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
    sync::{Arc, atomic::AtomicBool},
};

use sqlx::{Pool, Postgres};
use tokio::sync::RwLock;

use crate::{
    config::Config,
    eco_movement::api::client::{self, EcoMovementClient},
    slack::Slack,
};

#[derive(Clone)]
pub struct State {
    pub inner: Arc<InnerState>,
}

pub struct InnerState {
    pub database_pool: Pool<Postgres>,
    pub eco_movement_api: client::EcoMovementClient,
    pub http_client: reqwest::Client,
    pub config: Config,
    pub slack: Option<Slack>,
    pub tokens: RwLock<HashSet<String>>,
    import_lock: AtomicBool,
}

impl State {
    pub fn new(database_pool: Pool<Postgres>, config: Config) -> State {
        let slack = match (&config.slack_token, &config.slack_channel) {
            (Some(token), Some(channel)) => Slack::new(token.clone(), channel.clone()).ok(),
            _ => None,
        };
        let eco_movement_api = EcoMovementClient::new(
            config.eco_movement_api_url.clone(),
            &config.eco_movement_api_key,
        );

        State {
            inner: Arc::new(InnerState {
                eco_movement_api,
                database_pool,
                config,
                slack,
                http_client: reqwest::Client::new(),
                tokens: Default::default(),
                import_lock: AtomicBool::new(false),
            }),
        }
    }

    pub fn lock(&self) -> Option<ImportLock> {
        if self.is_import_locked() {
            None
        } else {
            self.import_lock
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Some(ImportLock {
                lock: &self.import_lock,
            })
        }
    }

    pub fn is_import_locked(&self) -> bool {
        self.import_lock.load(std::sync::atomic::Ordering::SeqCst)
    }
}

pub struct ImportLock<'a> {
    lock: &'a AtomicBool,
}

impl Drop for ImportLock<'_> {
    fn drop(&mut self) {
        self.lock.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Deref for State {
    type Target = Arc<InnerState>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for State {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
