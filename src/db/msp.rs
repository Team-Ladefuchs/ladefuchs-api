use sqlx::Postgres;

use crate::{charge_price_api::response::MSPApiResult, db::price::ChargePrice};

pub async fn save(
    name: &str,
    msp_id: uuid::Uuid,
    transaction: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<i32, sqlx::error::Error> {
    let row = sqlx::query_file!("sql/insert_update/msp.sql", msp_id, name.trim())
        .fetch_one(transaction)
        .await?;
    Ok(row.id)
}

pub async fn save_all(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    msps: &[MSPApiResult],
    vehicle_id: i32,
    cpo_id: i32,
) -> Result<(), sqlx::Error> {
    let msps = msps
        .iter()
        .filter(|m| !m.attributes.tariff_name.to_lowercase().contains("business"));
    for msp in msps {
        let msp_id = save(&msp.attributes.provider, msp.id, transaction).await?;
        let tarif_id = msp.into_tarif(vehicle_id, msp_id).save(transaction).await?;
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
