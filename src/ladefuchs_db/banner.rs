use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};
use sqlx::{Connection, PgConnection};

#[derive(Debug)]
pub enum BannerPathVersion {
    V2,
    V3,
}

pub enum BannerStatus {
    Active,
    Inactive,
    Deleted,
}

impl Display for BannerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BannerStatus::Active => write!(f, "active"),
            BannerStatus::Inactive => write!(f, "inactive"),
            BannerStatus::Deleted => write!(f, "deleted"),
        }
    }
}

pub mod v2 {
    use crate::api::banner::v2::Banner;

    use super::*;

    pub async fn get_all_banner(
        connection: &mut PgConnection,
        api_base_url: &url::Url,
        banner_version: BannerPathVersion,
        status: Option<BannerStatus>,
    ) -> Result<Vec<Banner>, sqlx::Error> {
        let rows = sqlx::query_file!(
            "sql/get/banner/link_banner.sql",
            status.map(|s| s.to_string())
        )
        .fetch_all(connection)
        .await?
        .into_iter()
        .map(|row| {
            let image_url = {
                let mut url = api_base_url.clone();

                if let Ok(mut path_segments) = url.path_segments_mut() {
                    match banner_version {
                        BannerPathVersion::V2 => {
                            path_segments.extend(["img", "banner", &row.checksum]);
                        }
                        BannerPathVersion::V3 => {
                            path_segments.extend(["image", &row.checksum]);
                        }
                    };
                }
                url
            };

            let link = {
                let mut url = api_base_url.clone();
                url.set_path("affiliate");
                url.query_pairs_mut().append_pair("url", &row.source);
                url.query_pairs_mut()
                    .append_pair("banner", &row.id.to_string());

                url
            };

            Banner {
                id: row.id,
                link,
                image: image_url,
                filename: PathBuf::from(row.image)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                is_affiliate: row.is_affiliate,
                frequency: row.frequency,
                updated: row.updated,
            }
        })
        .collect::<Vec<_>>();
        Ok(rows)
    }
}

pub async fn link_id(connection: &mut PgConnection, link: &url::Url) -> Option<i32> {
    let url_str = link.as_str();
    let link = match url_str.strip_suffix("/") {
        Some(url) => url,
        None => url_str,
    };
    sqlx::query_file!("sql/get/banner/single_link.sql", link)
        .fetch_optional(connection)
        .await
        .ok()
        .flatten()
        .map(|row| row.id)
}

pub async fn get_by_id(connection: &mut PgConnection, id: &uuid::Uuid) -> Option<(i32, String)> {
    sqlx::query_file!("sql/get/banner/link_banner_by_uuid.sql", id)
        .fetch_optional(connection)
        .await
        .ok()
        .flatten()
        .map(|row| (row.id, row.file_path))
}

pub async fn update_link_states(
    connection: &mut PgConnection,
    link_id: i32,
    platform: &PlatformType,
    banner_id: Option<i32>,
) -> Result<(), sqlx::Error> {
    let mut trx = connection.begin().await?;
    sqlx::query_file!(
        "sql/insert/link_states.sql",
        link_id,
        platform as _,
        banner_id
    )
    .execute(&mut *trx)
    .await?;
    trx.commit().await?;
    Ok(())
}

pub async fn add_banner_impression(
    connection: &mut PgConnection,
    banner_id: &uuid::Uuid,
    platform: &PlatformType,
) -> Result<(), sqlx::Error> {
    let mut transaction: sqlx::Transaction<sqlx::Postgres> = connection.begin().await?;

    sqlx::query_file!(
        "sql/insert/add_impression_banner.sql",
        banner_id,
        platform as _,
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    Ok(())
}

#[derive(sqlx::Type, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlatformType {
    #[serde(rename = "ios")]
    #[allow(clippy::upper_case_acronyms)]
    IOS,
    Android,
    Web,
}

impl From<&str> for PlatformType {
    fn from(user_agent: &str) -> Self {
        match user_agent.contains("Android") {
            true => Self::Android,
            false
                if user_agent.contains("iPhone")
                    || user_agent.contains("iPad")
                    || user_agent.contains("iPod") =>
            {
                Self::IOS
            }
            false => Self::Web,
        }
    }
}
