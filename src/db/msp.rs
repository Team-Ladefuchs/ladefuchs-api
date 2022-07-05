use crate::{charge_price_api::response::MSPApiResult, db::charge_price::ChargePrice};
use sqlx::Postgres;

use super::cpo_msp;

pub async fn save(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    msp_id: &uuid::Uuid,
    name: &str,
) -> Result<i32, sqlx::error::Error> {
    let normalized_name = normalize_name(name);
    match get_by_id_or_name(transaction, &msp_id, name.trim()).await? {
        Some(msp_id) => {
            update(transaction, msp_id, name.trim(), &normalized_name).await?;
            Ok(msp_id)
        }
        None => {
            let id = sqlx::query_file_scalar!(
                "sql/insert_update/msp.sql",
                msp_id,
                name.trim(),
                normalized_name
            )
            .fetch_one(transaction)
            .await?;
            Ok(id)
        }
    }
}

pub async fn save_all(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    msps: &[MSPApiResult],
    cpo_id: i32,
) -> Result<(), sqlx::Error> {
    let msps = msps
        .iter()
        .filter(|m| !m.attributes.tariff_name.to_lowercase().contains("business"));

    for msp in msps {
        let msp_id = save(transaction, &msp.id, &msp.attributes.provider).await?;
        let tariff_id = msp.into_tariff(msp_id).save(transaction).await?;
        cpo_msp::insert_update(transaction, &cpo_id, &msp_id).await?;
        let charge_prices = msp
            .attributes
            .charge_point_prices
            .iter()
            .filter(|tariff| tariff.price_distribution.kwh == Some(1.0));

        for tariff in charge_prices {
            tracing::debug!(provider=%msp.attributes.provider, price=%tariff.price, tariff=%msp.attributes.tariff_name, plug=%tariff.plug);
            let plug = &tariff.plug;
            ChargePrice {
                cpo_id,
                tariff_id,
                c_type: plug.into(),
                price: tariff.price,
                blocking_fee_start: tariff.blocking_fee_start.unwrap_or_default(),
            }
            .save(transaction)
            .await?
        }
    }
    Ok(())
}

pub async fn get_by_id_or_name(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    msp_id: &uuid::Uuid,
    name: &str,
) -> Result<Option<i32>, sqlx::error::Error> {
    let row = sqlx::query_file!("sql/get/msp_by_id_name.sql", msp_id, name)
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
                _ => Some(c),
            };
            if !c.is_whitespace() {
                white_space_mode = false
            }
            ret
        })
        .collect()
}
