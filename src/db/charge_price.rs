use super::operator::{self};
use crate::{
    api::{
        charge_condition::v2,
        charge_condition::v3::{self, TariffConditions},
        error::ApiError,
    },
    db::plug::ChargeType,
};
use chrono::Utc;
use paste::paste;
use sqlx::PgConnection;

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

macro_rules! get_charge_conditions {
    ($sql:expr, $fn_suffix:ident) => {
        paste! {
            pub async fn [<charge_conditions_ $fn_suffix>](
                connection: &mut PgConnection,
                operator_ids: &[uuid::Uuid],
                tariff_ids: &[uuid::Uuid],
                charge_mods: &[ChargeType],
            ) -> Result<v3::ChargeConditionResponse, sqlx::Error> {
                let mut charging_conditions = vec![];

                for operator_id in operator_ids {
                    let tariff_conditions = sqlx::query_file_as!(
                        v3::ChargeCondition,
                        $sql,
                        operator_id,
                        charge_mods as _,
                        tariff_ids,
                    )
                    .fetch_all(&mut *connection)
                    .await?;
                    charging_conditions.push(TariffConditions {
                        operator_id: operator_id.clone(),
                        tariff_conditions,
                    })
                }

                Ok(v3::ChargeConditionResponse {
                    last_updated_date: charging_conditions
                        .first()
                        .and_then(|tariff| tariff.tariff_conditions.first())
                        .map(|item| item.updated),
                    charging_conditions,
                })
            }
        }
    };
}

get_charge_conditions!("sql/get/charge_price/v3/conditions_custom.sql", custom);
get_charge_conditions!(
    "sql/get/charge_price/v3/conditions_by_network.sql",
    standard
);

pub async fn get_card_prices_by_operator<T>(
    connection: &mut PgConnection,
    operator_ids: Vec<uuid::Uuid>,
    domain: &url::Url,
    tariffs: &[uuid::Uuid],
) -> Result<v2::AllCard<T>, sqlx::Error>
where
    T: std::convert::From<v2::Card>,
{
    let mut operator_map = Vec::with_capacity(operator_ids.len());

    for operator in operator_ids {
        let cards = sqlx::query_file_as!(
            v2::Card,
            "sql/get/charge_price/v2/charge_prices_all_by_network.sql",
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

        operator_map.push(v2::ChargePriceMap { operator, ac, dc });
    }
    Ok(operator_map)
}

async fn get_cards_by_type(
    connection: &mut PgConnection,
    operator_id: i32,
    charge_type: &ChargeType,
    domain: &url::Url,
) -> Result<Vec<v2::Card>, sqlx::Error> {
    let cards = sqlx::query_file_as!(
        v2::Card,
        "sql/get/charge_price/v2/charge_prices_by_type.sql",
        operator_id,
        charge_type as _,
        domain.to_string()
    )
    .fetch_all(connection)
    .await?;

    Ok(cards)
}

pub async fn last_import_context(
    connection: &mut PgConnection,
    interval_time: Option<chrono::Duration>,
) -> Result<admin::ImportResult, sqlx::Error> {
    let row = sqlx::query_file!("sql/get/charge_price/last_import.sql")
        .fetch_one(connection)
        .await?;
    let last_import = row.last_import;

    let interval_time = interval_time.unwrap_or_else(|| chrono::Duration::hours(0));

    Ok(admin::ImportResult {
        prices: row.prices,
        last_import,
        next_import: Utc::now() + interval_time,
    })
}

pub async fn get_cards<T>(
    connection: &mut PgConnection,
    charge_type: &ChargeType,
    cpo_name: &str,
    domain: &url::Url,
) -> Result<Vec<T>, ApiError>
where
    T: From<v2::Card>,
{
    match operator::get_by_pub_id_or_name(connection, &cpo_name).await {
        Ok(cpo_id) => {
            let cards = get_cards_by_type(connection, cpo_id, charge_type, domain)
                .await?
                .into_iter()
                .map(T::from)
                .collect();
            Ok(cards)
        }
        Err(_) => Err(ApiError::OperatorNotFound(cpo_name.to_string())),
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

pub mod admin {
    use super::*;
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
}
