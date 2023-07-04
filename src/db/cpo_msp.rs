use sqlx::PgConnection;

async fn get(
    transaction: &mut PgConnection,
    cpo_id: &i32,
    msp_id: &i32,
) -> Result<Option<i32>, sqlx::Error> {
    sqlx::query_file_scalar!("sql/get/msp/msp_cpo.sql", cpo_id, msp_id)
        .fetch_optional(transaction)
        .await
}

pub async fn insert_update(
    transaction: &mut PgConnection,
    cpo_id: &i32,
    msp_id: &i32,
) -> Result<(), sqlx::Error> {
    let row = get(transaction, cpo_id, msp_id).await?;

    if row.is_none() {
        sqlx::query_file!("sql/insert/msp/add_msp_cpo.sql", cpo_id, msp_id)
            .execute(transaction)
            .await?;
    }
    Ok(())
}
