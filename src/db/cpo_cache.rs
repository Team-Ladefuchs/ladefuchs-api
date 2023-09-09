use chrono::Utc;
use serde::Serialize;
use sqlx::PgConnection;

use crate::charge_price_api::{client::ChargingStationsStatists, response::CompanyResult};

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

pub async fn clear(transaction: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/delete/cpo_cache.sql",)
        .execute(&mut *transaction)
        .await?;
    Ok(())
}

pub async fn save_all_operator(
    transaction: &mut PgConnection,
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

pub async fn get_by_network(
    connection: &mut PgConnection,
    network: &uuid::Uuid,
) -> Result<i32, sqlx::Error> {
    sqlx::query_file_scalar!("sql/get/cpo/cpo_cache_by_network.sql", network)
        .fetch_one(&mut *connection)
        .await
}

pub async fn search(
    connection: &mut PgConnection,
    query: &str,
) -> Result<Vec<CPOCache>, sqlx::Error> {
    sqlx::query_file_as!(CPOCache, "sql/get/cpo/search_cache.sql", query)
        .fetch_all(connection)
        .await
}
