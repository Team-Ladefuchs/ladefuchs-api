use std::sync::Arc;

use crate::api::card::{self, CardV2};
use crate::api::error::ApiError;
use crate::api::CardV2List;
use crate::db::plug::ChargeType;
use chrono::Utc;
use futures_util::future;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use tokio::sync::Mutex;

use super::cpo::{self};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChargePrice {
    pub cpo_id: i32,
    pub tariff_id: i32,
    pub c_type: ChargeType,
    pub price: f64,
    pub blocking_fee_start: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargePriceMap {
    operator: uuid::Uuid,
    ac: Vec<CardV2>,
    dc: Vec<CardV2>,
}

impl ChargePrice {
    pub async fn save(
        &self,
        transaction: &mut sqlx::Transaction<'_, Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        tracing::log::debug!("{:#?}", self);
        sqlx::query_file!(
            "sql/insert/charge_price.sql",
            self.cpo_id,
            self.tariff_id,
            self.c_type as ChargeType,
            self.price,
            self.blocking_fee_start
        )
        .execute(transaction)
        .await?;
        Ok(())
    }
}

pub async fn get_all_prices_by_cpo(
    connection: &mut PoolConnection<Postgres>,
    cpo_ids: &[uuid::Uuid],
    domain: &url::Url,
) -> Result<CardV2List, sqlx::Error> {
    let ids = cpo::get_by_all_by_network(connection, cpo_ids).await?;

    let connection_wrap = Arc::new(Mutex::new(connection));

    let tasks = ids.into_iter().map(|(cpo_id, network)| {
        get_ac_and_dc_cards(connection_wrap.clone(), network, cpo_id, domain)
    });

    let prices = future::try_join_all(tasks).await?;
    Ok(prices)
}

async fn get_ac_and_dc_cards(
    connection: Arc<Mutex<&mut PoolConnection<Postgres>>>,
    network: uuid::Uuid,
    cpo_id: i32,
    domain: &reqwest::Url,
) -> Result<ChargePriceMap, sqlx::Error> {
    let mut connection = connection.lock().await;
    let cards = ChargePriceMap {
        operator: network,
        ac: get_prices_by_type(&mut connection, cpo_id, &ChargeType::AC, domain).await?,
        dc: get_prices_by_type(&mut connection, cpo_id, &ChargeType::DC, domain).await?,
    };
    Ok(cards)
}

async fn get_prices_by_type(
    connection: &mut PoolConnection<Postgres>,
    cpo_id: i32,
    charge_type: &ChargeType,
    domain: &url::Url,
) -> Result<Vec<CardV2>, sqlx::Error> {
    let cards = sqlx::query_file_as!(
        CardV2,
        "sql/get/charge_price/charge_prices_by_type.sql",
        cpo_id,
        charge_type as _,
        domain.to_string()
    )
    .fetch_all(connection)
    .await?;

    Ok(cards)
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub prices: Option<i64>,
    pub last_import: Option<chrono::DateTime<Utc>>,
    pub next_import: chrono::DateTime<Utc>,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminImportResult {
    pub status: ImportStatus,
    pub import_result: Option<ImportResult>,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ImportStatus {
    Waiting,
    InProgress,
}

impl From<bool> for ImportStatus {
    fn from(value: bool) -> Self {
        match value {
            true => Self::InProgress,
            false => Self::Waiting,
        }
    }
}

pub async fn import_metadata(
    connection: &mut PoolConnection<Postgres>,
    interval_time: Option<chrono::Duration>,
) -> Result<ImportResult, sqlx::Error> {
    let row = sqlx::query_file!("sql/get/charge_price/last_import.sql")
        .fetch_one(connection)
        .await?;
    let last_import = row.last_import;

    let interval_time = interval_time.unwrap_or_else(|| chrono::Duration::hours(0));

    Ok(ImportResult {
        prices: row.prices,
        last_import,
        next_import: Utc::now() + interval_time,
    })
}

pub async fn get<T>(
    connection: &mut PoolConnection<Postgres>,
    charge_type: &ChargeType,
    cpo_name: &str,
    domain: &url::Url,
) -> Result<Vec<T>, ApiError>
where
    T: From<card::CardV2>,
{
    match cpo::get_by_pub_id_or_name(connection, &cpo_name).await {
        Some(cpo_id) => {
            let cards = get_prices_by_type(connection, cpo_id, charge_type, domain)
                .await?
                .into_iter()
                .map(T::from)
                .collect();
            Ok(cards)
        }
        None => Err(ApiError::CpoNotFound(cpo_name.to_string())),
    }
}

pub async fn clear_all(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/delete/all_prices.sql")
        .execute(&mut *transaction)
        .await?;
    Ok(())
}

pub async fn clear_by_cpo(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    cpo_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/delete/prices_for_cpo.sql", cpo_id)
        .execute(&mut *transaction)
        .await?;

    Ok(())
}
