use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use sqlx::{Pool, Postgres};

use crate::{charge_price_api::client::ChargePriceAPI, config::Config, slack::Slack, timer};

#[derive(Clone)]
pub struct State {
    pub inner: Arc<InnerState>,
}

pub struct InnerState {
    pub charge_price_api: ChargePriceAPI,
    pub database_pool: Pool<Postgres>,
    pub config: Config,
    pub slack: Option<Slack>,
    pub timer: timer::Timer,
}

impl State {
    pub fn new(database_pool: Pool<Postgres>, config: Config, timer: timer::Timer) -> State {
        let slack = match (&config.slack_token, &config.slack_channel) {
            (Some(token), Some(channel)) => Slack::new(token.clone(), channel.clone()).ok(),
            _ => None,
        };
        let charge_price_api = ChargePriceAPI::new(
            config.charge_price_api_url.to_string(),
            &config.charge_price_api_key,
        );

        State {
            inner: Arc::new(InnerState {
                charge_price_api,
                database_pool,
                config,
                slack,
                timer,
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
