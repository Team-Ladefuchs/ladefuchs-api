use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use strum_macros::IntoStaticStr;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr, Deserialize, Serialize, sqlx::Type,
)]
#[serde(rename_all(serialize = "lowercase"))]
#[sqlx(type_name = "chargetype")]
pub enum ChargeType {
    #[serde(alias = "ac")]
    AC,
    #[serde(alias = "dc")]
    DC,
}

impl fmt::Display for ChargeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
