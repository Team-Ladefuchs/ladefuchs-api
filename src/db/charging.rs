use std::fmt::Display;

use serde::{Deserialize, Serialize};

use strum_macros::EnumString;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    Deserialize,
    Serialize,
    sqlx::Type,
)]
#[strum(serialize_all = "UPPERCASE")]
pub enum ChargeType {
    AC,
    DC,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString, Serialize, Deserialize,
)]
#[strum(serialize_all = "UPPERCASE")]
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
