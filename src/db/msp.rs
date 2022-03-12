use sqlx::Postgres;

// #[derive(Debug, Clone, Deserialize)]
// pub struct MSP {
//     id: i32,
//     name: String,
//     is_enabled: bool,
// }

pub async fn save(
    name: &str,
    msp_id: uuid::Uuid,
    transaction: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<i32, sqlx::error::Error> {
    let row = sqlx::query_file!("sql/insert_update/msp.sql", msp_id, &name)
        .fetch_one(transaction)
        .await?;
    Ok(row.id)
}
