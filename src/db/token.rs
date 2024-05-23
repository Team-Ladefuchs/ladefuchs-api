use std::collections::HashSet;

use sqlx::{PgConnection, PgPool};

pub async fn get_api_token(database_pool: &PgPool) -> Result<HashSet<String>, sqlx::Error> {
    let mut connection: sqlx::pool::PoolConnection<sqlx::Postgres> =
        database_pool.acquire().await?;
    let results = sqlx::query_file!("sql/get/tokens.sql")
        .fetch_all(&mut *connection)
        .await?;
    Ok(results.into_iter().map(|row| row.value).collect())
}

pub async fn get_random_token(connection: &mut PgConnection) -> Result<String, sqlx::Error> {
    let result = sqlx::query_file_scalar!("sql/get/single_token.sql")
        .fetch_one(&mut *connection)
        .await?;
    Ok(result)
}
