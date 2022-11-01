use super::{plug::Plug, PGPoolConnection};
use crate::inc_sql;
use chrono::serde::ts_seconds;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Acquire;
use sqlx::{postgres, Row};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct CPO {
    pub id: i32,
    pub network: uuid::Uuid,
    pub pub_network: uuid::Uuid,
    pub is_enabled: bool,
    pub slug_name: String,
    pub name: String,
    pub hide: bool,
    pub supported_types: BTreeMap<Plug, Meta>,
    pub updated: chrono::DateTime<Utc>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Meta {
    pub power: i32,
    pub expect: i32,
}

pub async fn get_with(
    connection: &mut PGPoolConnection,
    filter: Filter,
) -> Result<Vec<CPO>, sqlx::Error> {
    let cpos = get_all(connection)
        .await?
        .into_iter()
        .filter(|item| match filter {
            Filter::All => true,
            Filter::Enabled => item.is_enabled == true,
            Filter::Disabled => item.is_enabled == false,
        })
        .collect::<_>();
    Ok(cpos)
}

pub async fn get_all(connection: &mut PGPoolConnection) -> Result<Vec<CPO>, sqlx::Error> {
    let cpos = sqlx::query(inc_sql!("get/cpo/cpos"))
        .fetch_all(connection)
        .await?
        .iter()
        .map(CPO::from)
        .collect::<_>();

    Ok(cpos)
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
            updated: row.get("updated"),
            supported_types: charge_map,
            url: row.try_get("url").ok(),
            hide: row.get("hide"),
        }
    }
}

pub async fn hide_with_no_prices(
    connection: &mut PGPoolConnection,
) -> Result<Vec<String>, sqlx::Error> {
    let mut transaction = connection.begin().await?;
    let mut cpo_names = vec![];
    let cpos = sqlx::query_file!("sql/get/inactive_cpos.sql")
        .fetch_all(&mut *transaction)
        .await?;

    let cpo_count = sqlx::query_file_scalar!("sql/get/cpo/cpo_enabled_count.sql")
        .fetch_one(&mut transaction)
        .await?
        .unwrap_or_default() as usize;

    // do not hide all cpos
    if cpo_count == cpos.len() {
        return Ok(cpo_names);
    }

    for row in cpos {
        cpo_names.push(row.slug_name);
        sqlx::query_file!("sql/update/hide_cpo.sql", row.id)
            .execute(&mut *transaction)
            .await?;
    }
    transaction.commit().await?;

    Ok(cpo_names)
}

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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorV2 {
    pub identifier: uuid::Uuid,
    pub display_name: String,
    pub types: Vec<String>,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
}

pub async fn all_operators(
    connection: &mut PGPoolConnection,
) -> Result<Vec<Operator>, sqlx::Error> {
    operator_by(connection, true, true).await
}

pub async fn enabled_operators(
    connection: &mut PGPoolConnection,
) -> Result<Vec<Operator>, sqlx::Error> {
    operator_by(connection, true, false).await
}

pub async fn disabled_operators(
    connection: &mut PGPoolConnection,
) -> Result<Vec<Operator>, sqlx::Error> {
    operator_by(connection, false, false).await
}

async fn operator_by(
    connection: &mut PGPoolConnection,
    is_enabled: bool,
    ignore_filter: bool,
) -> Result<Vec<Operator>, sqlx::Error> {
    sqlx::query_file_as!(
        Operator,
        "sql/get/cpo/operator.sql",
        is_enabled,
        ignore_filter
    )
    .fetch_all(connection)
    .await
}

pub async fn all_operators_v2(
    connection: &mut PGPoolConnection,
) -> Result<Vec<OperatorV2>, sqlx::Error> {
    operator_by2(connection, true, true).await
}

pub async fn enabled_operators_v2(
    connection: &mut PGPoolConnection,
) -> Result<Vec<OperatorV2>, sqlx::Error> {
    operator_by2(connection, true, false).await
}

pub async fn disabled_operators_v2(
    connection: &mut PGPoolConnection,
) -> Result<Vec<OperatorV2>, sqlx::Error> {
    operator_by2(connection, false, false).await
}

async fn operator_by2(
    connection: &mut PGPoolConnection,
    is_enabled: bool,
    ignore_filter: bool,
) -> Result<Vec<OperatorV2>, sqlx::Error> {
    sqlx::query_file_as!(
        OperatorV2,
        "sql/get/cpo/operatorV2.sql",
        is_enabled,
        ignore_filter
    )
    .fetch_all(connection)
    .await
}
