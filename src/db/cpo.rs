use serde::Deserialize;
use sqlx::{postgres, Row};
use std::collections::BTreeMap;

use crate::{api::cpo, inc_sql};

use super::{charging::Plug, MyPool};

#[derive(Debug, Clone)]
pub struct CPO {
    pub id: i32,
    pub network: uuid::Uuid,
    pub pub_network: uuid::Uuid,
    pub is_enabled: bool,
    pub slug_name: String,
    pub name: String,
    pub supported_types: BTreeMap<Plug, Meta>,
}

#[derive(Debug, Clone, Deserialize)]

pub struct Meta {
    pub power: i32,
    pub expect: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Extra {
    #[serde(rename = "powerAC")]
    pub power_ac: i32,
    #[serde(rename = "powerDC")]
    pub power_dc: i32,
    #[serde(rename = "expectAC")]
    pub expect_ac: i32,
    #[serde(rename = "expectDC")]
    pub expect_dc: i32,
}

pub async fn get_with(pool: &MyPool, filter: cpo::Mode) -> Result<Vec<CPO>, sqlx::Error> {
    let cpos = get_all(pool)
        .await?
        .into_iter()
        .filter(|item| match filter {
            cpo::Mode::All => true,
            cpo::Mode::Enabled => item.is_enabled == true,
            cpo::Mode::Disabled => item.is_enabled == false,
        })
        .collect::<_>();
    Ok(cpos)
}

pub async fn get_all(pool: &MyPool) -> Result<Vec<CPO>, sqlx::Error> {
    let cpos = sqlx::query(inc_sql!("get/all_cpos"))
        .fetch_all(pool)
        .await?
        .iter()
        .map(CPO::from)
        .collect::<_>();
    Ok(cpos)
}

impl From<&postgres::PgRow> for CPO {
    fn from(row: &postgres::PgRow) -> Self {
        let extra: Option<Extra> = match row.get::<Option<serde_json::Value>, _>("extra") {
            Some(v) => serde_json::from_value(v).unwrap(),
            None => None,
        };

        let mut charge_map = BTreeMap::new();

        if let Some(ext) = &extra {
            if ext.expect_ac > 0 {
                charge_map.insert(
                    Plug::TYPE2,
                    Meta {
                        power: ext.power_ac,
                        expect: ext.expect_ac,
                    },
                );
            }
            if ext.expect_dc > 0 {
                charge_map.insert(
                    Plug::CCS,
                    Meta {
                        power: ext.power_dc,
                        expect: ext.expect_dc,
                    },
                );
            }
        }

        CPO {
            id: row.get("id"),
            network: row.get("network"),
            is_enabled: row.get("is_enabled"),
            slug_name: row.get("slug_name"),
            name: row.get("name"),
            pub_network: row.get("pub_network"),
            supported_types: charge_map,
        }
    }
}
