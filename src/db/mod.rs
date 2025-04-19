pub mod app_metrics;
pub mod banner;
pub mod charge_price;
pub mod feedback;
pub mod image;
pub mod operator;
pub mod plug;
pub mod tariff;
pub mod token;
pub mod user;

use std::str::FromStr;
use std::time::Duration;

use sqlx::pool::PoolOptions;
use sqlx::postgres::Postgres;
use sqlx::{ConnectOptions, Pool};
use tracing::log;

pub async fn connect(
    url: &url::Url,
    database_pool_size: u32,
) -> Result<Pool<Postgres>, sqlx::Error> {
    let options = sqlx::postgres::PgConnectOptions::from_str(url.as_str())?
        .log_statements(log::LevelFilter::Error)
        .disable_statement_logging()
        .log_slow_statements(log::LevelFilter::Warn, Duration::from_secs(2));

    let pool = PoolOptions::new()
        .min_connections(database_pool_size)
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy_with(options);
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
