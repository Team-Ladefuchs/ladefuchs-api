use crate::{
    charge_price_api::response::ApiResponse,
    db::charge_price::ChargePrice,
    slack::{Slack, SlackClient},
};
use sqlx::{pool::PoolConnection, Postgres};

use super::cpo_msp;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Msp {
    id: uuid::Uuid,
    name: String,
    operator_id: uuid::Uuid,
}

pub async fn get_all(connection: &mut PoolConnection<Postgres>) -> Result<Vec<Msp>, sqlx::Error> {
    sqlx::query_file_as!(Msp, "sql/get/msp/all_msp.sql")
        .fetch_all(connection)
        .await
}

pub async fn save(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    name: &str,
) -> Result<i32, sqlx::error::Error> {
    let normalized_name = normalize_name(name);
    match get_by_name(transaction, name.trim()).await? {
        Some(msp_id) => {
            update(transaction, msp_id, name.trim(), &normalized_name).await?;
            Ok(msp_id)
        }
        None => {
            let id =
                sqlx::query_file_scalar!("sql/insert/msp/msp.sql", name.trim(), normalized_name)
                    .fetch_one(transaction)
                    .await?;
            Ok(id)
        }
    }
}

pub async fn save_all(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    responses: &[ApiResponse],
    slack: &Option<Slack>,
) -> Result<u64, sqlx::Error> {
    let mut prices_count = 0;

    let filter_list = sqlx::query_file!("sql/get/all_filter.sql")
        .fetch_all(&mut *transaction)
        .await?
        .into_iter()
        .filter_map(|row| {
            // maybe try regex set https://docs.rs/regex/latest/regex/struct.RegexSet.html (faster)
            regex::RegexBuilder::new(&row.value)
                .case_insensitive(true)
                .build()
                .ok()
        })
        .collect::<Vec<_>>();

    slack.reset_count();
    for response in responses {
        let only_kwh_msps = response
            .msps
            .iter()
            .filter(|msp| {
                msp.attributes
                    .charge_point_prices
                    .iter()
                    .all(|charge_price| charge_price.price_distribution.kwh == Some(1.0))
            })
            .filter(|msp| {
                filter_list.iter().all(|filter_item| {
                    let tariff_id = &msp.relationships.tariff.data.id;
                    // maybe to it cleaner
                    // filter tariff name or tariff id
                    !filter_item.is_match(&msp.attributes.tariff_name)
                        && !filter_item.is_match(&tariff_id.to_string())
                })
            });

        for msp in only_kwh_msps {
            let msp_id = save(transaction, &msp.attributes.provider).await?;
            let tariff_id = msp
                .into_tariff(msp_id)
                .save(transaction, &response.cpo_name, slack)
                .await?;
            cpo_msp::insert_update(transaction, &response.cpo_id, &msp_id).await?;

            for tariff in &msp.attributes.charge_point_prices {
                prices_count += 1;
                tracing::debug!(provider=%msp.attributes.provider, price=%tariff.price, tariff=%msp.attributes.tariff_name, plug=%tariff.plug);
                let plug = &tariff.plug;
                ChargePrice {
                    cpo_id: response.cpo_id,
                    tariff_id,
                    c_type: plug.into(),
                    price: tariff.price,
                    blocking_fee_start: tariff.blocking_fee_start.unwrap_or_default(),
                }
                .save(transaction)
                .await?
            }
        }
    }

    Ok(prices_count)
}

pub async fn get_by_name(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    name: &str,
) -> Result<Option<i32>, sqlx::error::Error> {
    let row = sqlx::query_file!("sql/get/msp/msp_by_id_name.sql", name)
        .fetch_optional(transaction)
        .await?;
    Ok(row.map(|r| r.id))
}

async fn update(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: i32,
    name: &str,
    legacy_id: &str,
) -> Result<(), sqlx::error::Error> {
    sqlx::query_file!("sql/update/msp.sql", id, name, legacy_id)
        .execute(transaction)
        .await?;
    Ok(())
}

fn normalize_name(id: &str) -> String {
    let mut white_space_mode = false;
    id.trim()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .map(|c| c.to_ascii_lowercase())
        .filter_map(|c| {
            let ret = match c {
                c if c.is_whitespace() && !white_space_mode => {
                    white_space_mode = true;
                    Some('_')
                }
                c if c.is_whitespace() => None,
                'ä' => Some('a'),
                'ü' => Some('u'),
                'ö' => Some('o'),
                'ß' => Some('s'),
                _ => Some(c),
            };
            if !c.is_whitespace() {
                white_space_mode = false
            }
            ret
        })
        .collect()
}
