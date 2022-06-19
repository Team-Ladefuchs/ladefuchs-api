use std::path::PathBuf;

use sqlx::{pool::PoolConnection, Acquire, Postgres};

use crate::api::card;

#[derive(Debug, Clone)]
pub struct CardImageContext {
    pub tarif_id: i32,
    pub image: CardImage,
    pub filename: String,
}

#[derive(Debug, Clone)]
pub struct CardImage {
    pub file_path: PathBuf,
    pub checksum: String,
    pub mime: mime::Mime,
}

pub async fn insert_or_update(
    connection: &mut PoolConnection<Postgres>,
    card: &CardImageContext,
) -> Result<(), sqlx::Error> {
    let mut transaction = connection.begin().await?;

    let row = sqlx::query_file!(
        "sql/insert_update/add_card_image.sql",
        card.image.file_path.to_str(),
        card.image.checksum,
        card.image.mime.as_ref(),
    )
    .fetch_one(&mut transaction)
    .await?;

    sqlx::query_file!("sql/update/tarif_image.sql", row.id, card.tarif_id)
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
        "sql/update/card_image_path.sql",
        old_path.to_str(),
        new_path.to_str(),
    )
    .fetch_one(&mut transaction)
    .await?;

    sqlx::query_file!("sql/update/tarif_internal_name.sql", filename, row.id,)
        .execute(&mut transaction)
        .await?;

    transaction.commit().await?;
    Ok(())
}

pub async fn get_by_checksum(
    connection: &mut PoolConnection<Postgres>,
    checksum: &str,
) -> Result<CardImage, sqlx::Error> {
    let row = sqlx::query_file!("sql/get/card_image.sql", checksum)
        .fetch_one(connection)
        .await?;

    let image = CardImage {
        checksum: row.checksum,
        file_path: PathBuf::try_from(row.file_path).unwrap_or_default(),
        mime: row.mime_type.parse().unwrap_or_else(|_| mime::IMAGE_JPEG),
    };

    Ok(image)
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

pub async fn get_all(
    connection: &mut PoolConnection<Postgres>,
    domain: &url::Url,
) -> Result<Vec<card::Image>, sqlx::error::Error> {
    let rows = sqlx::query_file_as!(card::Image, "sql/get/tarif_images.sql", domain.as_str())
        .fetch_all(connection)
        .await?;
    Ok(rows)
}
