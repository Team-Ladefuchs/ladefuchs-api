pub mod charge_price;
pub mod cpo;
pub mod msp;
pub mod plug;
pub mod tarif;
pub mod vehicle;

use std::str::FromStr;
use std::time::Duration;

use sqlx::pool::{PoolConnection, PoolOptions};
use sqlx::postgres::Postgres;
use sqlx::{ConnectOptions, Pool};
use tracing::log;

pub type PGPoolConnection = PoolConnection<Postgres>;
pub async fn connect(
    url: &url::Url,
    database_pool_size: u32,
) -> Result<Pool<Postgres>, sqlx::Error> {
    let mut options = sqlx::postgres::PgConnectOptions::from_str(url.as_str())?;

    options
        .log_statements(log::LevelFilter::Error)
        .disable_statement_logging()
        .log_slow_statements(log::LevelFilter::Warn, Duration::from_secs(1));

    let pool = PoolOptions::new()
        .min_connections(database_pool_size)
        .connect_timeout(Duration::from_secs(4))
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
