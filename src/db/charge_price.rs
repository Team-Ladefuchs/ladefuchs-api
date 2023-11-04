use chrono::Utc;
use sqlx::PgConnection;

use super::operator::{self};
use crate::{
    api::{card::v3, error::ApiError, AllCard},
    db::plug::ChargeType,
};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChargePrice {
    pub operator_id: i32,
    pub operator_network: uuid::Uuid,
    pub tariff_relation: uuid::Uuid,
    pub tariff_id: i32,
    pub c_type: ChargeType,
    pub price: f64,
    pub blocking_fee_start: i64,
    pub blocking_fee: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargePriceMap<T> {
    operator: uuid::Uuid,
    ac: Vec<T>,
    dc: Vec<T>,
}

impl ChargePrice {
    pub async fn save(&self, transaction: &mut PgConnection) -> Result<(), sqlx::error::Error> {
        tracing::log::debug!("{:#?}", self);
        sqlx::query_file!(
            "sql/insert/charge_price.sql",
            self.operator_id,
            self.tariff_id,
            self.c_type as ChargeType,
            self.price,
            self.blocking_fee_start,
            self.blocking_fee
        )
        .execute(transaction)
        .await?;
        Ok(())
    }
}

pub async fn get_all_prices_by_cpo<T>(
    connection: &mut PgConnection,
    operator_ids: Vec<uuid::Uuid>,
    domain: &url::Url,
    tariffs: &Vec<uuid::Uuid>,
) -> Result<AllCard<T>, sqlx::Error>
where
    T: std::convert::From<v3::Card>,
{
    let mut operator_map = Vec::with_capacity(operator_ids.len());

    for operator in operator_ids {
        let cards = sqlx::query_file_as!(
            v3::Card,
            "sql/get/charge_price/charge_prices_all_by_network.sql",
            operator,
            domain.to_string(),
            tariffs
        )
        .fetch_all(&mut *connection)
        .await?;

        let mut ac = vec![];
        let mut dc = vec![];

        for card in cards {
            match card.c_type {
                ChargeType::AC => {
                    ac.push(card.into());
                }
                ChargeType::DC => dc.push(card.into()),
            }
        }

        operator_map.push(ChargePriceMap { operator, ac, dc });
    }
    Ok(operator_map)
}

async fn get_prices_by_type(
    connection: &mut PgConnection,
    cpo_id: i32,
    charge_type: &ChargeType,
    domain: &url::Url,
) -> Result<Vec<v3::Card>, sqlx::Error> {
    let cards = sqlx::query_file_as!(
        v3::Card,
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
    connection: &mut PgConnection,
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
    connection: &mut PgConnection,
    charge_type: &ChargeType,
    cpo_name: &str,
    domain: &url::Url,
) -> Result<Vec<T>, ApiError>
where
    T: From<v3::Card>,
{
    match operator::get_by_pub_id_or_name(connection, &cpo_name).await {
        Some(cpo_id) => {
            let cards = get_prices_by_type(connection, cpo_id, charge_type, domain)
                .await?
                .into_iter()
                .map(T::from)
                .collect();
            Ok(cards)
        }
        None => Err(ApiError::OperatorNotFound(cpo_name.to_string())),
    }
}

pub async fn clear_all(transaction: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/delete/all_prices.sql")
        .execute(&mut *transaction)
        .await?;
    Ok(())
}

pub async fn clear_by_operator(
    transaction: &mut PgConnection,
    operator_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/delete/prices_for_operator.sql", operator_id)
        .execute(&mut *transaction)
        .await?;

    Ok(())
}

pub async fn save_alle_prices(
    transaction: &mut PgConnection,
    charge_prices: Vec<ChargePrice>,
) -> Result<(), sqlx::Error> {
    for charge_price in &charge_prices {
        charge_price.save(transaction).await?;
    }
    Ok(())
}
