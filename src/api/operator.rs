use crate::db::{self};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub enum Filter {
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Operator {
    pub name: String,
    pub identifier: String,
    pub display_name: String,
}
