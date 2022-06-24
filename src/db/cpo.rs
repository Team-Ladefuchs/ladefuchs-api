use crate::{api::operator, inc_sql};
use serde::Deserialize;
use sqlx::Acquire;
use sqlx::{postgres, Row};
use std::collections::BTreeMap;

use super::{plug::Plug, PGPoolConnection};

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

pub async fn get_with(
    connection: &mut PGPoolConnection,
    filter: operator::Filter,
) -> Result<Vec<CPO>, sqlx::Error> {
    let cpos = get_all(connection)
        .await?
        .into_iter()
        .filter(|item| match filter {
            operator::Filter::All => true,
            operator::Filter::Enabled => item.is_enabled == true,
            operator::Filter::Disabled => item.is_enabled == false,
        })
        .collect::<_>();
    Ok(cpos)
}

pub async fn get_all(connection: &mut PGPoolConnection) -> Result<Vec<CPO>, sqlx::Error> {
    let cpos = sqlx::query(inc_sql!("get/cpos"))
        .fetch_all(connection)
        .await?
        .iter()
        .map(CPO::from)
        .collect::<_>();

    Ok(cpos)
}

pub async fn get_operators<T>(
    connection: &mut PGPoolConnection,
    filter: operator::Filter,
) -> Result<Vec<T>, sqlx::Error>
where
    T: From<CPO>,
{
    let operators = get_with(connection, filter)
        .await?
        .into_iter()
        .map(|item| T::from(item))
        .collect();
    Ok(operators)
}

impl From<&postgres::PgRow> for CPO {
    fn from(row: &postgres::PgRow) -> Self {
        let mut charge_map = BTreeMap::new();

        let power_ac = row.get("power_ac");
        let power_dc = row.get("power_dc");

        let expect_ac = row.get("expect_ac");
        let expect_dc = row.get("expect_dc");

        if expect_ac > 0 {
            charge_map.insert(
                Plug::TYPE2,
                Meta {
                    power: power_ac,
                    expect: expect_ac,
                },
            );
        }

        if expect_dc > 0 {
            charge_map.insert(
                Plug::CCS,
                Meta {
                    power: power_dc,
                    expect: expect_dc,
                },
            );
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

pub async fn disable_with_no_prices(
    connection: &mut PGPoolConnection,
) -> Result<Vec<String>, sqlx::Error> {
    let mut transaction = connection.begin().await?;
    let mut cpo_names = vec![];
    let cpos = sqlx::query_file!("sql/get/inactive_cpos.sql")
        .fetch_all(&mut *transaction)
        .await?;

    let cpo_count = sqlx::query_file_scalar!("sql/get/cpo_enabled_count.sql")
        .fetch_one(&mut transaction)
        .await?
        .unwrap_or_default() as usize;

    if cpo_count == cpos.len() {
        return Ok(cpo_names);
    }

    for row in cpos {
        cpo_names.push(row.name);
        sqlx::query_file!("sql/update/disable_cpo.sql", row.id)
            .execute(&mut *transaction)
            .await?;
    }
    transaction.commit().await?;

    Ok(cpo_names)
}
