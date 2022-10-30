use crate::api::card::{self, CardV2};
use crate::db::plug::ChargeType;
use chrono::Utc;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChargePrice {
    pub cpo_id: i32,
    pub tariff_id: i32,
    pub c_type: ChargeType,
    pub price: f64,
    pub blocking_fee_start: i64,
}

impl ChargePrice {
    pub async fn save(
        &self,
        transaction: &mut sqlx::Transaction<'_, Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        tracing::log::debug!("{:#?}", self);
        sqlx::query_file!(
            "sql/insert_update/charge_price.sql",
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

async fn get_prices(
    connection: &mut PoolConnection<Postgres>,
    cpo_name: &str,
    charge_type: &ChargeType,
    domain: &url::Url,
) -> Result<Vec<CardV2>, sqlx::Error> {
    let cards = sqlx::query_file_as!(
        CardV2,
        "sql/get/charge_prices.sql",
        cpo_name,
        charge_type as _,
        domain.to_string()
    )
    .fetch_all(connection)
    .await?;

    Ok(cards)
}

// TODO to camelCase
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub prices: Option<i64>,
    pub last_import: Option<chrono::DateTime<Utc>>,
    pub next_import: Option<chrono::DateTime<Utc>>,
}

pub async fn import_metadata(
    connection: &mut PoolConnection<Postgres>,
    offset_hours: u8,
) -> Result<ImportResult, sqlx::Error> {
    let row = sqlx::query_file!("sql/get/last_import.sql")
        .fetch_one(connection)
        .await?;
    let last_import = row.last_import;

    Ok(ImportResult {
        prices: row.prices,
        last_import,
        next_import: last_import.map(|time| time + crate::importer::hours(offset_hours)),
    })
}

pub async fn get<T>(
    connection: &mut PoolConnection<Postgres>,
    charge_type: &ChargeType,
    cpo_name: &str,
    domain: &url::Url,
) -> Result<Vec<T>, sqlx::Error>
where
    T: From<card::CardV2>,
{
    let cards = get_prices(connection, cpo_name, charge_type, domain)
        .await?
        .into_iter()
        .map(T::from)
        .collect();
    Ok(cards)
}

pub async fn clear(transaction: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/delete/prices.sql")
        .execute(transaction)
        .await?;
    Ok(())
}
