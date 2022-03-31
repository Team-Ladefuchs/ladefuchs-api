pub mod card;
pub mod charging;
pub mod cpo;
pub mod msp;
pub mod tarif;
pub mod vehicle;

use std::str::FromStr;
use std::time::Duration;

use sqlx::pool::PoolOptions;
use sqlx::postgres::Postgres;
use sqlx::{ConnectOptions, Pool};
use tracing::log;

use crate::charge_price_api::response::{AllChargePrices, MSPApiResult};

pub type MyPool = Pool<Postgres>;

pub async fn connect(url: &url::Url) -> Result<MyPool, sqlx::Error> {
    let mut options = sqlx::postgres::PgConnectOptions::from_str(url.as_str())?;

    options
        .log_statements(log::LevelFilter::Error)
        .disable_statement_logging()
        .log_slow_statements(log::LevelFilter::Warn, Duration::from_secs(1));

    let pool = PoolOptions::new()
        .min_connections(16)
        .connect_lazy_with(options.to_owned());
    migrate(&pool).await?;
    Ok(pool)
}

pub async fn migrate<T>(pool: &Pool<T>) -> Result<(), sqlx::Error>
where
    <T as sqlx::Database>::Connection: sqlx::migrate::Migrate,
    T: sqlx::Database,
{
    sqlx::migrate!().run(pool).await?;
    Ok(())
}

#[macro_export]
macro_rules! inc_sql {
    ($e:expr) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/sql/", $e, ".sql"))
    };
}

pub async fn import(results: AllChargePrices, pool: &MyPool) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    for api_result in results {
        save_msps(
            &mut transaction,
            &api_result.msps,
            api_result.vehicle_id,
            api_result.cpo_id,
        )
        .await?
    }
    // futures_util::future::try_join_all(results);
    transaction.commit().await?;
    Ok(())
}

pub async fn save_msps(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    msps: &[MSPApiResult],
    vehicle_id: i32,
    cpo_id: i32,
) -> Result<(), sqlx::Error> {
    let msps = msps
        .iter()
        .filter(|m| !m.attributes.tariff_name.to_lowercase().contains("business"));
    for msp in msps {
        let msp_id = msp::save(&msp.attributes.provider, msp.id, transaction).await?;
        let tarif_id = msp.into_tarif(vehicle_id, msp_id).save(transaction).await?;
        let charge_prices = msp
            .attributes
            .charge_point_prices
            .iter()
            .filter(|tarif| tarif.price_distribution.kwh == Some(1.0));
        for tarif in charge_prices {
            tracing::info!(provider=%msp.attributes.provider, price=%tarif.price, tarif=%msp.attributes.tariff_name, plug=%tarif.plug);
            card::Card {
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

// #[cfg(test)]
// mod tests {
//     use model::ChargeType;

//     use super::*;
//     use crate::config;

//     #[tokio::test]
//     async fn test_get_cpo() {
//         let config = config::read_config().unwrap();
//         let pool = connect(&config.database_url).await.unwrap();
//         let mut conn = pool.acquire().await.unwrap();
//         let cpos = get_cpos(&mut *conn).await.unwrap();
//         let ionity = cpos.iter().find(|cpo| cpo.name == "ionity").unwrap();
//         assert!(ionity.charge_types.get(&ChargeType::DC).is_some());
//         assert!(ionity.charge_types.get(&ChargeType::AC).is_none());
//         assert!(!cpos.is_empty())
//     }
// }
