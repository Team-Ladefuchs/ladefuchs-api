use sqlx::{Connection, PgConnection};

pub async fn new_admin_account(
    connection: &mut PgConnection,
    username: &str,
    password: &str,
) -> Result<(), eyre::Error> {
    let row = sqlx::query_file!("sql/get/admin_by_name.sql", &username)
        .fetch_optional(&mut *connection)
        .await?;

    if row.is_some() {
        return Ok(());
    }

    let pwd_hash = bcrypt::hash(&password, 10)?;

    if username.is_empty() {
        return Ok(());
    }

    let mut transaction = connection.begin().await?;
    sqlx::query_file!("sql/insert/add_admin.sql", username, pwd_hash)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(())
}
