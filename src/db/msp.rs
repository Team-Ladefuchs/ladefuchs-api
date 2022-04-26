use crate::{charge_price_api::response::MSPApiResult, db::charge_price::ChargePrice};
use sqlx::Postgres;

pub async fn save(
    name: &str,
    msp_id: uuid::Uuid,
    transaction: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<i32, sqlx::error::Error> {
    let row = sqlx::query_file!(
        "sql/insert_update/msp.sql",
        msp_id,
        name.trim(),
        normalize_name(name)
    )
    .fetch_one(transaction)
    .await?;
    Ok(row.id)
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
        let msp_id = save(&msp.attributes.provider, msp.id, transaction).await?;
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

fn normalize_name(id: &str) -> String {
    let mut white_space_mode = false;
    id.chars()
        .filter(|c| c.is_alphabetic() || c.is_whitespace())
        .map(|c| c.to_ascii_lowercase())
        .filter_map(|c| {
            let ret = match c {
                ' ' if !white_space_mode => {
                    white_space_mode = true;
                    Some('_')
                }
                ' ' => None,
                _ => Some(c),
            };
            if !c.is_whitespace() {
                white_space_mode = false
            }
            ret
        })
        .collect()
}
