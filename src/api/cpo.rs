use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::{self, charging::ChargeType, cpo::get_with, MyPool};

#[derive(Debug, Clone, Deserialize)]
pub enum Mode {
    #[serde(alias = "all")]
    All,
    #[serde(alias = "enabled")]
    Enabled,
    #[serde(alias = "disabled")]
    Disabled,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V1 {
    id: Uuid,
    identifier: String,
    display_name: String,
    current_types: Vec<ChargeType>,
}

impl From<&db::cpo::CPO> for V1 {
    fn from(value: &db::cpo::CPO) -> Self {
        Self {
            id: value.pub_network,
            identifier: value.name.to_lowercase(),
            display_name: value.slug_name.clone(),
            current_types: value
                .supported_types
                .clone()
                .into_iter()
                .map(|(plug, _)| plug.into())
                .collect(),
        }
    }
}

pub async fn get_operators(
    filter: Mode,
    pool: &MyPool,
) -> Result<HashMap<String, V1>, sqlx::Error> {
    let operators = get_with(pool, filter)
        .await?
        .iter()
        .map(|item| (item.name.to_lowercase(), V1::from(item)))
        .collect();
    Ok(operators)
}
