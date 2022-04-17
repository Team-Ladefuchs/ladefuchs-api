use crate::db::{self, cpo::get_with, MyPool};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub enum Mode {
    #[serde(alias = "all")]
    All,
    #[serde(alias = "enabled")]
    Enabled,
    #[serde(alias = "disabled")]
    Disabled,
}

impl From<&db::cpo::CPO> for Operator {
    fn from(value: &db::cpo::CPO) -> Self {
        let lowercase_name = value.name.to_lowercase();
        Self {
            identifier: format!("cpo-{}", lowercase_name),
            name: lowercase_name,
            display_name: value.slug_name.clone(),
        }
    }
}

pub async fn get_operators(filter: Mode, pool: &MyPool) -> Result<Vec<Operator>, sqlx::Error> {
    let operators = get_with(pool, filter)
        .await?
        .iter()
        .map(|item| Operator::from(item))
        .collect();
    Ok(operators)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Operator {
    pub name: String,
    pub identifier: String,
    pub display_name: String,
}
