use std::fmt::Display;

use serde::{Deserialize, Serialize};

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
pub enum ChargeType {
    #[serde(alias = "ac")]
    AC,
    #[serde(alias = "dc")]
    DC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Plug {
    TYPE2,
    CCS,
}

impl From<Plug> for ChargeType {
    fn from(p: Plug) -> Self {
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
