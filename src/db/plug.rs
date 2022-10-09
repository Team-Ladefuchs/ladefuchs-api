use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum_macros::IntoStaticStr;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    IntoStaticStr,
    Deserialize,
    Serialize,
    sqlx::Type,
)]
#[serde(rename_all(serialize = "lowercase"))]
#[sqlx(type_name = "ChargeType", rename_all = "UPPERCASE")]
pub enum ChargeType {
    #[serde(alias = "ac")]
    AC,
    #[serde(alias = "dc")]
    DC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all(serialize = "lowercase"))]
pub enum Plug {
    #[serde(alias = "type2")]
    TYPE2,
    #[serde(alias = "ccs")]
    CCS,
}

impl From<&Plug> for ChargeType {
    fn from(p: &Plug) -> Self {
        match p {
            Plug::TYPE2 => ChargeType::AC,
            Plug::CCS => ChargeType::DC,
        }
    }
}

impl Display for Plug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
