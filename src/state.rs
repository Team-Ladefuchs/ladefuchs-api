use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use sqlx::{Pool, Postgres};

use crate::config::Config;

#[derive(Clone, Debug)]
pub struct State {
    pub inner: Arc<InnerState>,
}

#[derive(Clone, Debug)]
pub struct InnerState {
    pub database_pool: Pool<Postgres>,
    pub config: Config,
}

impl State {
    pub fn new(database_pool: Pool<Postgres>, config: Config) -> State {
        State {
            inner: Arc::new(InnerState {
                database_pool,
                config,
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
