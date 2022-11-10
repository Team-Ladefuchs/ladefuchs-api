use super::plug::ChargeType;
use super::PGPoolConnection;

use chrono::serde::ts_seconds;
use chrono::Utc;
use paste::paste;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, Postgres, Transaction};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CPO {
    pub id: i32,
    pub network: uuid::Uuid,
    pub pub_network: uuid::Uuid,
    pub is_enabled: bool,
    pub slug_name: String,
    pub name: String,
    pub hide: bool,
    pub supported_types: Vec<ChargeType>,
    pub updated: chrono::DateTime<Utc>,
    pub power_ac: i32,
    pub power_dc: i32,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Meta {
    pub power: i32,
}

impl CPO {
    pub async fn insert_or_update(
        &self,
        connection: &mut PGPoolConnection,
    ) -> Result<i32, sqlx::Error> {
        let types: Vec<String> = self.supported_types.iter().map(|t| t.to_string()).collect();
        let mut transaction = connection.begin().await?;
        let cpo_id = match get_by_network(&mut transaction, self.network).await {
            Some(id) => {
                sqlx::query_file_scalar!(
                    "sql/update/cpo.sql",
                    id,
                    self.name,
                    self.slug_name,
                    self.is_enabled,
                    types as Vec<String>,
                    self.power_ac,
                    self.power_dc
                )
                .fetch_one(&mut *transaction)
                .await?
            }
            None => {
                sqlx::query_file_scalar!(
                    "sql/insert/add_cpo.sql",
                    self.name,
                    self.slug_name,
                    self.network,
                    self.is_enabled,
                    types as _,
                    self.power_ac,
                    self.power_dc
                )
                .fetch_one(&mut *transaction)
                .await?
            }
        };

        transaction.commit().await?;
        Ok(cpo_id)
    }
}

pub async fn get_by_internal_id(
    connection: &mut PGPoolConnection,
    cpo_id: i32,
) -> Result<CPO, sqlx::Error> {
    sqlx::query_file_as!(CPO, "sql/get/cpo/cpo_by_internal_id.sql", cpo_id)
        .fetch_one(connection)
        .await
}

pub async fn has_no_prices(
    connection: &mut PGPoolConnection,
    cpo_id: i32,
) -> Result<bool, sqlx::Error> {
    let ret = sqlx::query_file_scalar!("sql/get/cpo/cpo_has_price.sql", cpo_id)
        .fetch_optional(connection)
        .await?
        .is_none();
    Ok(ret)
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

pub async fn get_by_network(
    transaction: &mut Transaction<'_, Postgres>,
    network: uuid::Uuid,
) -> Option<i32> {
    sqlx::query_file_scalar!("sql/get/cpo/cpo_by_network.sql", network)
        .fetch_one(&mut *transaction)
        .await
        .ok()
}

pub async fn get_by_pub_id_or_name(connection: &mut PGPoolConnection, name: &str) -> Option<i32> {
    sqlx::query_file_scalar!("sql/get/cpo/cpo_by_id_or_name.sql", name)
        .fetch_one(&mut *connection)
        .await
        .ok()
}

pub async fn get_all(connection: &mut PGPoolConnection) -> Result<Vec<CPO>, sqlx::Error> {
    let cpos = sqlx::query_file_as!(CPO, "sql/get/cpo/cpos.sql")
        .fetch_all(connection)
        .await?;

    Ok(cpos)
}

pub async fn toggle_hidden(
    transaction: &mut Transaction<'_, Postgres>,
    cpos: &[CPO],
) -> Result<(), sqlx::Error> {
    for cpo in cpos {
        sqlx::query_file!("sql/update/set_cpo_visibility.sql", false, cpo.id)
            .execute(&mut *transaction)
            .await?;
    }
    Ok(())
}

pub async fn hide_with_no_prices(
    connection: &mut PGPoolConnection,
    all_cpos: &[CPO],
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

    toggle_hidden(&mut transaction, all_cpos).await?;

    for row in cpos {
        cpo_names.push(row.slug_name);
        sqlx::query_file!("sql/update/set_cpo_visibility.sql", true, row.id)
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
    pub types: Vec<ChargeType>,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
}

macro_rules! get_operators {
    ($type:ty, $sql:expr, $version:ident) => {
        paste! {
            pub async fn [<all_operators_ $version>](
                connection: &mut PGPoolConnection,
            ) -> Result<Vec<$type>, sqlx::Error> {
                [<operator_by_ $version>](connection, true, true).await
            }

            pub async fn [<enabled_operators_ $version>](
                connection: &mut PGPoolConnection,
            ) -> Result<Vec<$type>, sqlx::Error> {
                [<operator_by_ $version>](connection, true, false).await
            }

            pub async fn [<disabled_operators_ $version>](
                connection: &mut PGPoolConnection,
            ) -> Result<Vec<$type>, sqlx::Error> {
                [<operator_by_ $version>](connection, false, false).await
            }

            async fn [<operator_by_ $version>](
                connection: &mut PGPoolConnection,
                is_enabled: bool,
                ignore_filter: bool,
            ) -> Result<Vec<$type>, sqlx::Error> {
                sqlx::query_file_as!(
                    $type,
                    $sql,
                    is_enabled,
                    ignore_filter
                )
                .fetch_all(connection)
                .await
            }
        }
    };
}

get_operators!(OperatorV2, "sql/get/cpo/operatorV2.sql", v2);
get_operators!(Operator, "sql/get/cpo/operator.sql", v1);
