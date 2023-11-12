use std::path::PathBuf;

use ::chrono::serde::ts_seconds;
use chrono::Utc;
use serde::Serialize;
use sqlx::{postgres, Connection, PgConnection};

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

#[derive(Debug)]
pub enum BannerPathVersion {
    V2,
    V3,
}

pub mod v2 {
    use super::*;

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Banner {
        pub link: url::Url,
        pub image: url::Url,
        pub frequency: i16,
        pub is_affiliate: bool,
        pub id: uuid::Uuid,
        #[serde(with = "ts_seconds")]
        pub updated: chrono::DateTime<Utc>,
        pub filename: String,
    }

    pub async fn get_all_banner(
        connection: &mut PgConnection,
        api_base_url: &url::Url,
        banner_version: BannerPathVersion,
    ) -> Result<Vec<Banner>, sqlx::Error> {
        let rows = sqlx::query_file!("sql/get/banner/link_banner.sql")
            .fetch_all(connection)
            .await?
            .into_iter()
            .map(|row| {
                let image_url = {
                    let mut url = url::Url::from(api_base_url.clone());

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
                    let mut url = url::Url::from(api_base_url.clone());
                    url.set_path("affiliate");
                    url.query_pairs_mut()
                        .append_pair("url", &escape_url(&row.source));
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

pub mod v3 {

    use super::*;
    use crate::api::serialize_iso_8601;

    impl From<v2::Banner> for Banner {
        fn from(value: v2::Banner) -> Self {
            Self {
                affiliate_link_url: value.link,
                image_url: value.image,
                frequency: value.frequency,
                is_affiliate: value.is_affiliate,
                id: value.id,
                last_updated_date: value.updated,
            }
        }
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Banner {
        pub affiliate_link_url: url::Url,
        pub image_url: url::Url,
        pub frequency: i16,
        pub is_affiliate: bool,
        pub id: uuid::Uuid,
        #[serde(serialize_with = "serialize_iso_8601")]
        pub last_updated_date: chrono::DateTime<Utc>,
    }
}

fn escape_url(url: &str) -> String {
    utf8_percent_encode(url, NON_ALPHANUMERIC).to_string()
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ClicksPerDay {
    #[serde(with = "ts_seconds")]
    pub day: chrono::DateTime<Utc>,
    pub clicks: i64,
}

pub async fn banner_click_statistics(
    connection: &mut PgConnection,
    days: i32,
    link_id: i32,
) -> Result<Vec<ClicksPerDay>, sqlx::Error> {
    let interval = postgres::types::PgInterval {
        months: 0,
        days,
        microseconds: 0,
    };
    let rows = sqlx::query_file_as!(
        ClicksPerDay,
        "sql/get/banner/banner_statistics.sql",
        interval,
        link_id
    )
    .fetch_all(connection)
    .await?;

    Ok(rows)
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ThgClickSummery {
    pub last_thirty_days: i64,
    pub last_seven_days: i64,
    pub average_weekly: i64,
    pub total_by_platform: ThgPlatformTotal,
    pub total: Option<i64>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ThgPlatformTotal {
    pub android: i64,
    pub ios: i64,
    pub web: i64,
}

pub async fn banner_click_summary(
    connection: &mut PgConnection,
    link_id: i32,
) -> Result<ThgClickSummery, sqlx::Error> {
    let mut interval = postgres::types::PgInterval {
        months: 0,
        days: 7,
        microseconds: 0,
    };
    let last_seven_days = sqlx::query_file_scalar!(
        "sql/get/banner/banner_statistics_last_days.sql",
        interval,
        link_id
    )
    .fetch_one(&mut *connection)
    .await?;

    interval.days = 30;
    let last_thirty_days = sqlx::query_file_scalar!(
        "sql/get/banner/banner_statistics_last_days.sql",
        interval,
        link_id
    )
    .fetch_one(&mut *connection)
    .await?;

    let average_weekly =
        sqlx::query_file_scalar!("sql/get/banner/banner_average_weekly.sql", link_id)
            .fetch_one(&mut *connection)
            .await?;

    let total = sqlx::query_file_scalar!("sql/get/banner/banner_total_by_id.sql", link_id)
        .fetch_one(&mut *connection)
        .await?;

    let total_by_platform = sqlx::query_file_as!(
        ThgPlatformTotal,
        "sql/get/banner/banner_statistics_platform.sql",
        link_id
    )
    .fetch_one(&mut *connection)
    .await?;

    Ok(ThgClickSummery {
        total,
        last_thirty_days,
        last_seven_days,
        average_weekly,
        total_by_platform,
    })
}

pub async fn get_id_by_name(
    connection: &mut PgConnection,
    filename: &str,
) -> Result<i32, sqlx::Error> {
    sqlx::query_file_scalar!("sql/get/banner/banner_by_name.sql", filename)
        .fetch_one(connection)
        .await
}

pub async fn set_image(
    transaction: &mut PgConnection,
    banner_id: i32,
    image_id: Option<i32>,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/update/image_banner_id.sql", image_id, banner_id)
        .execute(transaction)
        .await?;
    Ok(())
}

#[derive(sqlx::Type, Debug, Clone, Serialize)]
pub enum PlatformType {
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
