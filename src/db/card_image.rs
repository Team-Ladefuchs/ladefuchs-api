use std::path::PathBuf;

use chrono::{DateTime, Utc};
use sqlx::{pool::PoolConnection, Acquire, Postgres};

use crate::api::card;

#[derive(Debug, Clone)]
pub struct CardImageContext {
    pub tariff_id: i32,
    pub image: CardImage,
    pub filename: String,
    pub updated: DateTime<Utc>,
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
    let path = card.image.file_path.to_str();
    let row = sqlx::query_file_scalar!("sql/get/card_image_by_path.sql", path)
        .fetch_optional(&mut transaction)
        .await?;
    let image_id = match row {
        Some(id) => {
            sqlx::query_file!(
                "sql/update/tariff_image.sql",
                id,
                path,
                card.image.checksum,
                card.image.mime.as_ref(),
                card.updated,
            )
            .execute(&mut transaction)
            .await?;
            Some(id)
        }
        None => {
            sqlx::query_file_scalar!(
                "sql/insert/add_card_image.sql",
                path,
                card.image.checksum,
                card.image.mime.as_ref(),
                card.updated
            )
            .fetch_one(&mut transaction)
            .await?
        }
    };

    sqlx::query_file!("sql/update/image_tariff_id.sql", image_id, card.tariff_id)
        .execute(&mut transaction)
        .await?;

    transaction.commit().await?;
    Ok(())
}

pub async fn update_name_path(
    connection: &mut PoolConnection<Postgres>,
    old_path: &PathBuf,
    new_path: &PathBuf,
    filename: &str,
) -> Result<(), sqlx::Error> {
    let mut transaction = connection.begin().await?;
    let result = sqlx::query_file!(
        "sql/update/card_image_path.sql",
        old_path.to_str(),
        new_path.to_str(),
    )
    .fetch_optional(&mut transaction)
    .await?;

    match result {
        Some(row) => {
            sqlx::query_file!("sql/update/tariff_internal_name.sql", filename, row.id)
                .execute(&mut transaction)
                .await?;

            transaction.commit().await?;
        }
        _ => tracing::info!(msg = "Could not rename file", path = ?old_path),
    }

    Ok(())
}

pub async fn get_by_checksum(
    connection: &mut PoolConnection<Postgres>,
    checksum: &str,
) -> Result<CardImage, sqlx::Error> {
    let row = sqlx::query_file!("sql/get/tariff/tariff_image_by_checksum.sql", checksum)
        .fetch_one(connection)
        .await?;

    let image = CardImage {
        checksum: row.checksum,
        file_path: PathBuf::try_from(row.file_path).unwrap_or_default(),
        mime: row.mime_type.parse().unwrap_or_else(|_| mime::IMAGE_JPEG),
    };

    Ok(image)
}

pub async fn soft_delete(
    connection: &mut PoolConnection<Postgres>,
    path: &PathBuf,
) -> Result<(), sqlx::Error> {
    let path_str = path.to_str();
    let row = sqlx::query_file_scalar!("sql/get/card_image_by_path.sql", path_str)
        .fetch_optional(&mut *connection)
        .await?;

    if let Some(id) = row {
        tracing::debug!(event = "soft_delete", id, ?path);
        let mut transaction = connection.begin().await?;
        sqlx::query_file!("sql/update/soft_delete_image.sql", id)
            .execute(&mut transaction)
            .await?;

        transaction.commit().await?;
    }

    Ok(())
}

pub async fn delete_marked(connection: &mut PoolConnection<Postgres>) -> Result<(), sqlx::Error> {
    let mut transaction = connection.begin().await?;
    sqlx::query_file!("sql/delete/delete_marked.sql")
        .execute(&mut transaction)
        .await?;
    transaction.commit().await
}

pub async fn get_ad_hoc(transaction: &mut sqlx::Transaction<'_, Postgres>) -> Option<i32> {
    let row = sqlx::query_file_scalar!("sql/get/tariff/tariff_ad_hoc_image.sql")
        .fetch_one(transaction)
        .await
        .ok();
    row
}

pub async fn get_all(
    connection: &mut PoolConnection<Postgres>,
    domain: &url::Url,
) -> Result<Vec<card::Image>, sqlx::error::Error> {
    let rows = sqlx::query_file_as!(
        card::Image,
        "sql/get/tariff/tariff_images.sql",
        domain.as_str()
    )
    .fetch_all(connection)
    .await?;
    Ok(rows)
}
