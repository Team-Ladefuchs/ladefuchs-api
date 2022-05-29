use crate::{charge_price_api::response::MSPApiResult, db::charge_price::ChargePrice};
use sqlx::Postgres;

pub async fn save(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    msp_id: &uuid::Uuid,
    name: &str,
) -> Result<i32, sqlx::error::Error> {
    let normalized_name = normalize_name(name);
    match get_by_id(transaction, &msp_id).await? {
        Some(msp_id) => {
            update(transaction, msp_id, &normalized_name).await?;
            Ok(msp_id)
        }
        None => {
            let row = sqlx::query_file!(
                "sql/insert_update/msp.sql",
                msp_id,
                name.trim(),
                normalized_name
            )
            .fetch_one(transaction)
            .await?;
            Ok(row.id)
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
        let tarif_id = msp.into_tarif(msp_id).save(transaction).await?;
        let charge_prices = msp
            .attributes
            .charge_point_prices
            .iter()
            .filter(|tarif| tarif.price_distribution.kwh == Some(1.0));

        for tarif in charge_prices {
            tracing::info!(provider=%msp.attributes.provider, price=%tarif.price, tarif=%msp.attributes.tariff_name, plug=%tarif.plug);
            ChargePrice {
                cpo_id,
                tarif_id,
                c_type: tarif.plug.into(),
                price: tarif.price,
                blocking_fee_start: tarif.blocking_fee_start.unwrap_or_default(),
            }
            .save(transaction)
            .await?
        }
    }
    Ok(())
}

pub async fn get_by_id(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    msp_id: &uuid::Uuid,
) -> Result<Option<i32>, sqlx::error::Error> {
    let row = sqlx::query_file!("sql/get/msp_by_id.sql", msp_id,)
        .fetch_optional(transaction)
        .await?;
    Ok(row.map(|r| r.id))
}

async fn update(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: i32,
    name: &str,
) -> Result<(), sqlx::error::Error> {
    sqlx::query_file!("sql/update/msp.sql", id, name)
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
