use std::path::PathBuf;

use chrono::{DateTime, Utc};
use sqlx::{Connection, PgConnection};

use crate::file_watcher;

#[derive(Debug, Clone)]
pub struct ImageContext {
    pub source_id: i32,
    pub image: Image,
    pub filename: String,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Image {
    pub file_path: PathBuf,
    pub checksum: String,
    pub mime: mime::Mime,
}

pub async fn insert_or_update(
    transaction: &mut PgConnection,
    card: &ImageContext,
) -> Result<Option<i32>, sqlx::Error> {
    let path = card.image.file_path.to_str();
    let row = sqlx::query_file_scalar!("sql/get/image_by_path.sql", path)
        .fetch_optional(&mut *transaction)
        .await?;
    let image_id = match row {
        Some(id) => {
            sqlx::query_file!(
                "sql/update/image/image_metadata.sql",
                id,
                path,
                card.image.checksum,
                card.image.mime.as_ref(),
                card.updated,
            )
            .execute(&mut *transaction)
            .await?;
            Some(id)
        }
        None => {
            sqlx::query_file_scalar!(
                "sql/insert/image/add_image.sql",
                path,
                card.image.checksum,
                card.image.mime.as_ref(),
                card.updated
            )
            .fetch_one(&mut *transaction)
            .await?
        }
    };

    Ok(image_id)
}

pub async fn update_name_path(
    transaction: &mut PgConnection,
    old_path: &PathBuf,
    new_path: &PathBuf,
) -> Result<Option<i32>, sqlx::Error> {
    tracing::debug!(?old_path, ?new_path, "update file path");
    sqlx::query_file_scalar!(
        "sql/update/card_image_path.sql",
        old_path.to_str(),
        new_path.to_str(),
    )
    .fetch_optional(transaction)
    .await
}

pub async fn get_by_checksum(
    connection: &mut PgConnection,
    checksum: &str,
) -> Result<Image, sqlx::Error> {
    let row = sqlx::query_file!("sql/get/tariff/tariff_image_by_checksum.sql", checksum)
        .fetch_one(connection)
        .await?;

    let image = Image {
        checksum: row.checksum,
        file_path: PathBuf::try_from(row.file_path).unwrap_or_default(),
        mime: row.mime_type.parse().unwrap_or_else(|_| mime::IMAGE_JPEG),
    };

    Ok(image)
}

pub async fn soft_delete(connection: &mut PgConnection, path: &PathBuf) -> Result<(), sqlx::Error> {
    let path_str = path.to_str();
    let row = sqlx::query_file_scalar!("sql/get/image_by_path.sql", path_str)
        .fetch_optional(&mut *connection)
        .await?;

    if let Some(id) = row {
        tracing::debug!(event = "soft_delete", id, ?path);
        let mut transaction = connection.begin().await?;
        sqlx::query_file!("sql/update/soft_delete_image.sql", id)
            .execute(&mut *transaction)
            .await?;

        transaction.commit().await?;
    }

    Ok(())
}

pub async fn delete_marked(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
    let mut transaction = connection.begin().await?;
    sqlx::query_file!("sql/delete/delete_marked.sql")
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await
}

pub async fn get_ad_hoc(transaction: &mut sqlx::PgConnection) -> Option<i32> {
    let row = sqlx::query_file_scalar!("sql/get/tariff/tariff_ad_hoc_image.sql")
        .fetch_one(transaction)
        .await
        .ok();
    row
}

pub async fn update_image_file_name(
    connection: &mut PgConnection,
    internal_name: &str,
    image_id: i32,
    name_prefix: Option<&str>,
) -> Result<(), eyre::Error> {
    if let Some(mut path) = get_path_by_id(connection, image_id).await? {
        let file_name_without_ext = file_watcher::parse_filename(&path)?;
        if file_name_without_ext.eq(&internal_name) {
            return Ok(());
        }
        let current_path = path.clone();
        path.set_file_name(
            name_prefix
                .map(|prefix| format!("{}_{}", prefix, &internal_name))
                .unwrap_or_else(|| internal_name.to_string()),
        );
        path.set_extension(current_path.extension().unwrap_or_default());
        tokio::fs::rename(current_path, path).await?;
    }

    Ok(())
}

pub async fn get_path_by_id(
    connection: &mut PgConnection,
    id: i32,
) -> Result<Option<PathBuf>, sqlx::error::Error> {
    let ret = sqlx::query_file!("sql/get/image/image_by_id.sql", id)
        .fetch_optional(connection)
        .await?
        .map(|p| PathBuf::from(p.file_path));
    Ok(ret)
}

pub mod v3 {

    use crate::api::image::v3::{GenericImage, RelationType};

    use super::*;

    macro_rules! generic_image {
        ($row:expr, $relation_type:expr, $domain:expr) => {
            GenericImage {
                relation_id: $row.relation_id,
                relation_type: $relation_type,
                image_url: format_url(&$domain, &$row.blake3sum),
                blake3sum: $row.blake3sum,
                last_updated_date: $row.last_updated_date,
            }
        };
    }

    pub async fn get_all(
        connection: &mut PgConnection,
        domain: &url::Url,
    ) -> Result<Vec<GenericImage>, sqlx::error::Error> {
        let tariff_images = sqlx::query_file!("sql/get/image/v3/tariff_image.sql")
            .fetch_all(&mut *connection)
            .await?
            .into_iter()
            .map(|row| generic_image!(row, RelationType::Tariff, domain));

        let banner_images = sqlx::query_file!("sql/get/image/v3/banner_image.sql")
            .fetch_all(&mut *connection)
            .await?
            .into_iter()
            .map(|row| generic_image!(row, RelationType::Banner, domain));

        let operator_images = sqlx::query_file!("sql/get/image/v3/operator_image.sql")
            .fetch_all(connection)
            .await?
            .into_iter()
            .map(|row| generic_image!(row, RelationType::Operator, domain));

        Ok(operator_images
            .chain(tariff_images)
            .chain(banner_images)
            .collect::<Vec<_>>())
    }

    fn format_url(domain: &url::Url, blake3sum: &str) -> url::Url {
        let mut domain = domain.clone();
        if let Ok(mut path) = domain.path_segments_mut() {
            path.extend(["image", blake3sum]);
        }

        domain
    }
}
