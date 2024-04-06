use crate::{
    api::operator::{v1, v2, v3},
    charge_price_api::{client::ChargingStationsStatists, response::CompanyResult},
};

use super::plug::ChargeType;
use chrono::Utc;
use once_cell::sync::Lazy;
use paste::paste;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, PgConnection};

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

pub mod admin {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Operator {
        pub id: i32,
        pub network: uuid::Uuid,
        pub pub_network: uuid::Uuid,
        pub standard: bool,
        pub slug_name: String,
        pub name: String,
        pub hide: bool,
        pub supported_types: Vec<ChargeType>,
        pub updated: chrono::DateTime<Utc>,
        pub power_ac: i32,
        pub power_dc: i32,
        pub sum_plug_count: i32,
        pub ccs_plug_count: i32,
        pub type2_plug_count: i32,
        pub image: Option<i32>,
        pub url: Option<String>,
    }

    impl Operator {
        pub async fn update(&mut self, connection: &mut PgConnection) -> Result<(), sqlx::Error> {
            let types: Vec<String> = self.supported_types.iter().map(|t| t.to_string()).collect();
            self.name = normalize_internal_name(&self.slug_name);

            let mut transaction: sqlx::Transaction<sqlx::Postgres> = connection.begin().await?;
            sqlx::query_file_scalar!(
                "sql/update/operator/operator.sql",
                self.network,
                self.name,
                self.slug_name,
                self.standard,
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

    pub async fn get_all(connection: &mut PgConnection) -> Result<Vec<Operator>, sqlx::Error> {
        let operators = sqlx::query_file_as!(Operator, "sql/get/operator/admin/all_operators.sql")
            .fetch_all(connection)
            .await?;

        Ok(operators)
    }

    pub async fn get_by_internal_name_or_network(
        connection: &mut PgConnection,
        network: &uuid::Uuid,
        internal_name: &str,
    ) -> Result<Option<Operator>, sqlx::Error> {
        sqlx::query_file_as!(
            Operator,
            "sql/get/operator/admin/operator_by_internal_network.sql",
            network,
            internal_name
        )
        .fetch_optional(connection)
        .await
    }

    pub async fn get_with(
        connection: &mut PgConnection,
        filter: Filter,
    ) -> Result<Vec<Operator>, sqlx::Error> {
        let operators = get_all(connection)
            .await?
            .into_iter()
            .filter(|item| match filter {
                Filter::All => true,
                Filter::Enabled => item.standard == true,
                Filter::Disabled => item.standard == false,
            })
            .collect::<_>();
        Ok(operators)
    }
}

fn normalize_internal_name(slug_name: &str) -> String {
    REGEX_INTERNAL_OPERATOR_NAME
        .replace_all(slug_name, "")
        .to_lowercase()
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
) -> Result<Vec<admin::Operator>, sqlx::Error> {
    sqlx::query_file_as!(admin::Operator, "sql/get/operator/admin/search.sql", query)
        .fetch_all(connection)
        .await
}

pub async fn add_or_update_operator(
    connection: &mut PgConnection,
    company: &CompanyResult,
) -> Result<(), sqlx::Error> {
    let internal_name = normalize_internal_name(&company.attributes.name);
    match admin::get_by_internal_name_or_network(connection, &company.id, &internal_name).await? {
        Some(mut operator) => {
            operator.url = company.attributes.url.clone();
            if !operator.standard {
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

pub async fn get_by_pub_id_or_name(
    connection: &mut PgConnection,
    name: &str,
) -> Result<i32, sqlx::Error> {
    sqlx::query_file_scalar!("sql/get/operator/operator_by_id_or_name.sql", name)
        .fetch_one(&mut *connection)
        .await
}

pub async fn get_standard_with_no_prices(
    connection: &mut PgConnection,
) -> Result<Vec<String>, sqlx::Error> {
    let operators_names =
        sqlx::query_file_scalar!("sql/get/operator/import/inactive_operators.sql")
            .fetch_all(&mut *connection)
            .await?;
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

macro_rules! get_operators_enabled {
    ($type:ty, $sql:expr, $version:ident) => {
        paste! {
            pub async fn [<enabled_operators_ $version>](
                connection: &mut PgConnection,
                domain: &str,
            ) -> Result<Vec<$type>, sqlx::Error> {
                [<operator_by_ $version>](connection, true, domain).await
            }

            async fn [<operator_by_ $version>](
                connection: &mut PgConnection,
                standard: bool,
                domain: &str,
            ) -> Result<Vec<$type>, sqlx::Error> {
                sqlx::query_file_as!(
                    $type,
                    $sql,
                    standard,
                    domain
                )
                .fetch_all(connection)
                .await
            }
        }
    };
}

macro_rules! get_all_operators {
    ($type:ty, $sql:expr, $version:ident) => {
        paste! {
            pub async fn [<all_operators_ $version>](
                connection: &mut PgConnection,
                domain: &str,
            ) -> Result<Vec<$type>, sqlx::Error> {
                sqlx::query_file_as!(
                    $type,
                    $sql,
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
                [<operator_by_ $version>](connection, false, domain).await
            }
        }
    };
}
get_operators_enabled!(v3::Operator, "sql/get/operator/v3/operators.sql", v3);
get_all_operators!(v3::Operator, "sql/get/operator/v3/all_operators.sql", v3);

get_all_operators!(v2::Operator, "sql/get/operator/v2/all_operators.sql", v2);
get_operators_disabled!(v2::Operator, "sql/get/operator/v2/operators.sql", v2);
get_operators_enabled!(v2::Operator, "sql/get/operator/v2/operators.sql", v2);

get_all_operators!(v1::Operator, "sql/get/operator/v1/all_operators.sql", v1);
get_operators_disabled!(v1::Operator, "sql/get/operator/v1/operators.sql", v1);
get_operators_enabled!(v1::Operator, "sql/get/operator/v1/operators.sql", v1);
