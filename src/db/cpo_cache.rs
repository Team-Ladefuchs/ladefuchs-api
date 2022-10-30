use chrono::Utc;
use serde::Serialize;
use sqlx::Postgres;

use crate::charge_price_api::response::CompanyResult;

use super::PGPoolConnection;

#[derive(Debug, Clone, Serialize)]
pub struct CPOCache {
    pub id: i32,
    pub network: uuid::Uuid,
    pub slug_name: String,
    pub url: Option<String>,
    pub updated: chrono::DateTime<Utc>,
    pub is_added: bool,
}

pub async fn clear(transaction: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/delete/delete_marked.sql",)
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
            "sql/insert_update/add_cpo_cache.sql",
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
pub async fn search(
    connection: &mut PGPoolConnection,
    query: &str,
) -> Result<Vec<CPOCache>, sqlx::Error> {
    sqlx::query_file_as!(CPOCache, "sql/search/cpo_cache.sql", query)
        .fetch_all(connection)
        .await
}
