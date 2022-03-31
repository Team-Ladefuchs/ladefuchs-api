use serde::{Deserialize, Serialize};

use crate::db::{self, cpo::get_with, MyPool};

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
    identifier: String,
    name: String,
    display_name: String,
}

impl From<&db::cpo::CPO> for V1 {
    fn from(value: &db::cpo::CPO) -> Self {
        let lowercase_name = value.name.to_lowercase();
        Self {
            identifier: format!("cpo-{}", lowercase_name),
            display_name: value.slug_name.clone(),
            name: lowercase_name,
        }
    }
}

pub async fn get_operators(filter: Mode, pool: &MyPool) -> Result<Vec<V1>, sqlx::Error> {
    let operators = get_with(pool, filter)
        .await?
        .iter()
        .map(|item| V1::from(item))
        .collect();
    Ok(operators)
}
