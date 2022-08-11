use crate::db::{self, plug::ChargeType};
use chrono::{serde::ts_seconds, Utc};
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Operator {
    pub name: String,
    pub identifier: String,
    pub display_name: String,
}

impl From<db::cpo::CPO> for Operator {
    fn from(value: db::cpo::CPO) -> Self {
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
pub struct OperatorV2 {
    #[serde(skip)]
    pub name: String,
    pub identifier: uuid::Uuid,
    pub display_name: String,
    pub types: Vec<ChargeType>,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
}

impl From<db::cpo::CPO> for OperatorV2 {
    fn from(value: db::cpo::CPO) -> Self {
        Self {
            identifier: value.pub_network,
            name: value.name.to_lowercase(),
            display_name: value.slug_name.clone(),
            updated: value.updated,
            types: value
                .supported_types
                .iter()
                .map(|(plug, _)| plug.into())
                .collect(),
        }
    }
}
