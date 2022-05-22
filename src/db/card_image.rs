use std::path::{Path, PathBuf};

use sqlx::{pool::PoolConnection, Acquire, Postgres, Transaction};

#[derive(Debug, Clone)]
pub struct CardImage<'a> {
    pub tarif_id: i32,
    pub path: &'a Path,
    pub checksum: String,
    pub filename: String,
}

pub async fn insert_or_update(
    connection: &mut PoolConnection<Postgres>,
    card_image: &CardImage<'_>,
) -> Result<(), sqlx::Error> {
    let mut transaction = connection.begin().await?;
    sqlx::query_file!(
        "sql/insert_update/add_card_image.sql",
        card_image.tarif_id,
        card_image.path.to_str(),
        card_image.checksum,
        card_image.filename
    )
    .execute(&mut transaction)
    .await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn update_path(
    connection: &mut PoolConnection<Postgres>,
    old_path: &PathBuf,
    new_path: &PathBuf,
    filename: &str,
) -> Result<(), sqlx::Error> {
    let mut transaction = connection.begin().await?;
    let row = sqlx::query_file!(
        "sql/insert_update/card_image_path.sql",
        old_path.to_str(),
        new_path.to_str(),
        filename,
    )
    .fetch_one(&mut transaction)
    .await?;

    sqlx::query_file!(
        "sql/insert_update/tarif_internal_name.sql",
        row.tarif_id,
        filename,
    )
    .execute(&mut transaction)
    .await?;

    transaction.commit().await?;
    Ok(())
}

pub async fn delete(
    connection: &mut PoolConnection<Postgres>,
    path: &PathBuf,
) -> Result<(), sqlx::Error> {
    let mut transaction = connection.begin().await?;
    sqlx::query_file!("sql/delete/image.sql", path.to_str())
        .execute(&mut transaction)
        .await?;
    transaction.commit().await?;
    Ok(())
}
