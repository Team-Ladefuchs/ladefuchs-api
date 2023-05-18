use crate::api::card::{self, CardV2, CardV3};
use crate::api::error::ApiError;
use crate::api::AllCard;
use crate::db::plug::ChargeType;
use chrono::Utc;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;

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
    ac: Vec<CardV3>,
    dc: Vec<CardV3>,
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
    operator_ids: Vec<uuid::Uuid>,
    domain: &url::Url,
) -> Result<AllCard, sqlx::Error> {
    let mut cards = Vec::with_capacity(operator_ids.len());

    for operator in operator_ids {
        cards.push(get_all_prices(connection, operator, domain).await?);
    }
    Ok(cards)
}

pub async fn get_all_prices(
    connection: &mut PoolConnection<Postgres>,
    operator: uuid::Uuid,
    domain: &url::Url,
) -> Result<ChargePriceMap, sqlx::Error> {
    let cards = sqlx::query_file_as!(
        CardV3,
        "sql/get/charge_price/charge_prices_all.sql",
        operator,
        domain.to_string()
    )
    .fetch_all(connection)
    .await?;

    let mut ac = vec![];
    let mut dc = vec![];

    for card in cards {
        match card.c_type {
            ChargeType::AC => {
                ac.push(card);
            }
            ChargeType::DC => dc.push(card),
        }
    }

    Ok(ChargePriceMap { operator, ac, dc })
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
pub struct AdminImport {
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
