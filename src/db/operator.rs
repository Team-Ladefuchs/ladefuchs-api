use crate::charge_price_api::{client::ChargingStationsStatists, response::CompanyResult};

use super::plug::ChargeType;
use chrono::serde::ts_seconds;
use chrono::Utc;
use once_cell::sync::Lazy;
use paste::paste;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, PgConnection};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorIntern {
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
    pub image: Option<i32>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorSearchCache {
    pub id: i32,
    pub network: uuid::Uuid,
    pub slug_name: String,
    pub url: Option<String>,
    pub updated: chrono::DateTime<Utc>,
    pub cpo_id: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Meta {
    pub power: i32,
}

static REGEX_INTERNAL_OPERATOR_NAME: Lazy<regex::Regex> = Lazy::new(|| {
    regex::RegexBuilder::new(r#"[^A-Za-z0-9.+-]"#)
        .case_insensitive(true)
        .build()
        .unwrap()
});

impl OperatorIntern {
    pub async fn update(&self, connection: &mut PgConnection) -> Result<(), sqlx::Error> {
        let types: Vec<String> = self.supported_types.iter().map(|t| t.to_string()).collect();
        let mut transaction: sqlx::Transaction<sqlx::Postgres> = connection.begin().await?;
        let internal_name = normalize_internal_name(&self.slug_name);

        sqlx::query_file_scalar!(
            "sql/update/operator/operator.sql",
            self.network,
            internal_name,
            self.slug_name,
            self.is_enabled,
            types as Vec<String>,
            self.power_ac,
            self.power_dc
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }
}

fn normalize_internal_name(slug_name: &str) -> String {
    REGEX_INTERNAL_OPERATOR_NAME
        .replace_all(slug_name, "")
        .to_lowercase()
}

pub async fn get_by_internal_network_or_name(
    connection: &mut PgConnection,
    network: uuid::Uuid,
    internal_name: &str,
) -> Result<Option<OperatorIntern>, sqlx::Error> {
    sqlx::query_file_as!(
        OperatorIntern,
        "sql/get/operator/operator_by_internal_network.sql",
        network,
        internal_name
    )
    .fetch_optional(connection)
    .await
}

pub async fn has_no_prices(
    connection: &mut PgConnection,
    operator_id: i32,
) -> Result<bool, sqlx::Error> {
    let ret = sqlx::query_file_scalar!("sql/get/operator/operator_has_price.sql", operator_id)
        .fetch_optional(connection)
        .await?
        .is_none();
    Ok(ret)
}

pub async fn get_with(
    connection: &mut PgConnection,
    filter: Filter,
) -> Result<Vec<OperatorIntern>, sqlx::Error> {
    let operators = get_all(connection)
        .await?
        .into_iter()
        .filter(|item| match filter {
            Filter::All => true,
            Filter::Enabled => item.is_enabled == true,
            Filter::Disabled => item.is_enabled == false,
        })
        .collect::<_>();
    Ok(operators)
}

pub async fn update_charge_stations_statistics(
    transaction: &mut PgConnection,
    charge_stations: ChargingStationsStatists,
) -> Result<(), sqlx::Error> {
    for (id, station) in charge_stations.iter() {
        sqlx::query_file!(
            "sql/update/charge_stations_statistics.sql",
            id,
            station.ccs_count,
            station.type2_count
        )
        .execute(&mut *transaction)
        .await?;
    }
    Ok(())
}

pub async fn search(
    connection: &mut PgConnection,
    query: &str,
) -> Result<Vec<OperatorSearchCache>, sqlx::Error> {
    sqlx::query_file_as!(OperatorSearchCache, "sql/get/operator/search.sql", query)
        .fetch_all(connection)
        .await
}

pub async fn add_or_update_operator(
    connection: &mut PgConnection,
    company: &CompanyResult,
) -> Result<(), sqlx::Error> {
    let internal_name = normalize_internal_name(&company.attributes.name);
    match get_by_internal_network_or_name(connection, company.id, &internal_name).await? {
        Some(mut operator) => {
            operator.url = company.attributes.url.clone();
            operator.network = company.id;
            if !operator.is_enabled {
                operator.name = internal_name;
                operator.slug_name = company.attributes.name.clone();
            }
            operator.updated = company.attributes.updated_at;
            operator.update(connection).await?;
        }
        None => {
            let attributes = &company.attributes;
            sqlx::query_file!(
                "sql/insert/operator/add_operator.sql",
                company.id,
                internal_name,
                attributes.name,
                attributes.url,
                attributes.updated_at,
                false,
            )
            .execute(&mut *connection)
            .await?;
        }
    };
    Ok(())
}

pub async fn insert_or_update_companies(
    connection: &mut PgConnection,
    companies: &[CompanyResult],
) -> Result<(), sqlx::Error> {
    for company in companies {
        if let Err(error) = add_or_update_operator(connection, company).await {
            tracing::error!(
                task = "Error while import or update operator",
                ?error,
                ?company
            );
        }
    }
    Ok(())
}

pub async fn get_by_pub_id_or_name(connection: &mut PgConnection, name: &str) -> Option<i32> {
    sqlx::query_file_scalar!("sql/get/operator/operator_by_id_or_name.sql", name)
        .fetch_one(&mut *connection)
        .await
        .ok()
}

pub async fn get_all(connection: &mut PgConnection) -> Result<Vec<OperatorIntern>, sqlx::Error> {
    let operators = sqlx::query_file_as!(OperatorIntern, "sql/get/operator/all_operators.sql")
        .fetch_all(connection)
        .await?;

    Ok(operators)
}

pub async fn toggle_hidden(
    transaction: &mut PgConnection,
    operators: &[OperatorIntern],
) -> Result<(), sqlx::Error> {
    for cpo in operators {
        sqlx::query_file!(
            "sql/update/operator/set_operator_visibility.sql",
            false,
            cpo.id
        )
        .execute(&mut *transaction)
        .await?;
    }
    Ok(())
}

pub async fn delete_by_id(
    connection: &mut PgConnection,
    operator_id: i32,
) -> Result<(), sqlx::Error> {
    let mut transaction = connection.begin().await?;

    sqlx::query_file!("sql/delete/operator_by_id.sql", operator_id)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn hide_with_no_prices(
    connection: &mut PgConnection,
    all_operators: &[OperatorIntern],
) -> Result<Vec<String>, sqlx::Error> {
    let mut transaction = connection.begin().await?;
    let mut operators_names = vec![];
    let operators = sqlx::query_file!("sql/get/operator/inactive_operators.sql")
        .fetch_all(&mut *transaction)
        .await?;

    let operator_count = sqlx::query_file_scalar!("sql/get/operator/operator_enabled_count.sql")
        .fetch_one(&mut *transaction)
        .await?
        .unwrap_or_default() as usize;

    // do not hide all cpos
    if operator_count == operators.len() {
        return Ok(operators_names);
    }

    toggle_hidden(&mut transaction, all_operators).await?;

    for row in operators {
        operators_names.push(row.slug_name);
        sqlx::query_file!(
            "sql/update/operator/set_operator_visibility.sql",
            true,
            row.id
        )
        .execute(&mut *transaction)
        .await?;
    }
    transaction.commit().await?;

    Ok(operators_names)
}

pub async fn set_image(
    transaction: &mut PgConnection,
    operator_id: i32,
    image_id: Option<i32>,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "sql/update/operator/image_operator_id.sql",
        image_id,
        operator_id
    )
    .execute(transaction)
    .await?;
    Ok(())
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
    pub image: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorV3 {
    pub identifier: uuid::Uuid,
    pub display_name: String,
    pub types: Vec<ChargeType>,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
    pub image: Option<String>,
    pub is_default: bool,
    pub url: Option<String>,
}

#[warn(dead_code)]
macro_rules! get_operators {
    ($type:ty, $sql:expr, $version:ident) => {
        paste! {
            pub async fn [<all_operators_ $version>](
                connection: &mut PgConnection,
                domain: &str,
            ) -> Result<Vec<$type>, sqlx::Error> {
                [<operator_by_ $version>](connection, true, true, domain).await
            }

            pub async fn [<enabled_operators_ $version>](
                connection: &mut PgConnection,
                domain: &str,
            ) -> Result<Vec<$type>, sqlx::Error> {
                [<operator_by_ $version>](connection, true, false, domain).await
            }

            async fn [<operator_by_ $version>](
                connection: &mut PgConnection,
                is_enabled: bool,
                ignore_filter: bool,
                domain: &str,
            ) -> Result<Vec<$type>, sqlx::Error> {
                sqlx::query_file_as!(
                    $type,
                    $sql,
                    is_enabled,
                    ignore_filter,
                    domain
                )
                .fetch_all(connection)
                .await
            }
        }
    };
}

#[warn(dead_code)]
macro_rules! get_operators_disabled {
    ($type:ty, $sql:expr, $version:ident) => {
        paste! {
            pub async fn [<disabled_operators_ $version>](
                connection: &mut PgConnection,
                domain: &str,
            ) -> Result<Vec<$type>, sqlx::Error> {
                [<operator_by_ $version>](connection, false, false, domain).await
            }
        }
    };
}

get_operators!(OperatorV3, "sql/get/operator/operatorV3.sql", v3);
get_operators_disabled!(OperatorV2, "sql/get/operator/operatorV2.sql", v2);
get_operators_disabled!(Operator, "sql/get/operator/operator.sql", v1);
get_operators!(OperatorV2, "sql/get/operator/operatorV2.sql", v2);
get_operators!(Operator, "sql/get/operator/operator.sql", v1);
