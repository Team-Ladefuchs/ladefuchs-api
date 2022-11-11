use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock},
};

use sqlx::{Pool, Postgres};
use tokio::time;

use crate::{charge_price_api::client::ChargePriceAPI, config::Config, importer, slack::Slack};

#[derive(Clone, Debug)]
pub struct State {
    pub inner: Arc<InnerState>,
}
#[derive(Debug)]
pub struct InnerState {
    pub charge_price_api: ChargePriceAPI,
    pub database_pool: Pool<Postgres>,
    pub config: Config,
    pub slack: Option<Slack>,
    pub interval: RwLock<time::Interval>,
}

impl State {
    pub fn new(database_pool: Pool<Postgres>, config: Config) -> State {
        let slack = match (&config.slack_token, &config.slack_channel) {
            (Some(token), Some(channel)) => Slack::new(token.clone(), channel.clone()).ok(),
            _ => None,
        };
        let charge_price_api = ChargePriceAPI::new(
            config.charge_price_api_url.clone(),
            &config.charge_price_api_key,
        );

        let interval = time::interval(
            importer::hours(config.interval)
                .to_std()
                .expect("Invalid Duration"),
        );
        State {
            inner: Arc::new(InnerState {
                charge_price_api,
                database_pool,
                config,
                slack,
                interval: RwLock::new(interval),
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
