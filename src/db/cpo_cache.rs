use chrono::Utc;
use serde::Serialize;
use sqlx::Postgres;

use crate::charge_price_api::response::CompanyResult;

use super::PGPoolConnection;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CPOCache {
    pub id: i32,
    pub network: uuid::Uuid,
    pub slug_name: String,
    pub url: Option<String>,
    pub updated: chrono::DateTime<Utc>,
    pub cpo_id: Option<i32>,
}

pub async fn clear(transaction: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/delete/cpo_cache.sql",)
        .execute(&mut *transaction)
        .await?;
    Ok(())
}

pub async fn save_all(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    companies: &[CompanyResult],
) -> Result<(), sqlx::Error> {
    for company in companies {
        sqlx::query_file!(
            "sql/insert/cpo/add_cpo_cache.sql",
            company.id,
            company.attributes.name,
            company.attributes.url,
            company.attributes.updated_at
        )
        .execute(&mut *transaction)
        .await?;
    }

    Ok(())
}

pub async fn get_by_network(
    connection: &mut PGPoolConnection,
    network: &uuid::Uuid,
) -> Result<i32, sqlx::Error> {
    sqlx::query_file_scalar!("sql/get/cpo/cpo_cache_by_network.sql", network)
        .fetch_one(&mut *connection)
        .await
}

pub async fn search(
    connection: &mut PGPoolConnection,
    query: &str,
) -> Result<Vec<CPOCache>, sqlx::Error> {
    sqlx::query_file_as!(CPOCache, "sql/get/cpo/search_cache.sql", query)
        .fetch_all(connection)
        .await
}
