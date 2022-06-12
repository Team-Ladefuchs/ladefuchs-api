use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use sqlx::{Pool, Postgres};

use crate::{config::Config, slack::Slack};

#[derive(Clone, Debug)]
pub struct State {
    pub inner: Arc<InnerState>,
}

#[derive(Clone, Debug)]
pub struct InnerState {
    pub database_pool: Pool<Postgres>,
    pub config: Config,
    pub slack: Slack,
}

impl State {
    pub fn new(database_pool: Pool<Postgres>, config: Config) -> State {
        let slack = Slack::new(config.slack_token.clone(), config.slack_channel.clone());
        State {
            inner: Arc::new(InnerState {
                database_pool,
                config,
                slack,
            }),
        }
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
