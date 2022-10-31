use ::chrono::serde::ts_seconds;
use chrono::Utc;
use serde::Serialize;
use sqlx::{pool::PoolConnection, postgres, Acquire, Postgres};
use urlencoding::encode;

pub const BANNER_ROUTE: &str = "img/banner";

pub async fn get_all_banner(
    connection: &mut PoolConnection<Postgres>,
    api_url: &url::Url,
) -> Result<Vec<Banner>, sqlx::Error> {
    // let mut transaction = connection.begin().await?;
    let rows = sqlx::query_file!("sql/get/banner/link_banner.sql")
        .fetch_all(connection)
        .await?
        .into_iter()
        .filter_map(|row| {
            let url_str = format!(
                "{}affiliate?url={}&banner={}",
                api_url,
                encode(&row.source),
                row.id
            );

            let banner_url = format!("{}{}/{}", api_url, BANNER_ROUTE, row.id);
            match url::Url::parse(&url_str) {
                Ok(link) => Some(Banner {
                    id: row.id,
                    link,
                    image: banner_url,
                    filename: row.image,
                    is_affiliate: row.is_affiliate,
                    frequency: row.frequency,
                    updated: row.updated,
                }),
                Err(_) => None,
            }
        })
        .collect::<Vec<_>>();

    // transaction.commit().await?;
    Ok(rows)
}

pub async fn link_id(connection: &mut PoolConnection<Postgres>, link: &url::Url) -> Option<i32> {
    let url_str = link.as_str();
    let link = match url_str.strip_suffix("/") {
        Some(url) => url,
        None => url_str,
    };
    sqlx::query_file!("sql/get/single_link.sql", link)
        .fetch_optional(connection)
        .await
        .ok()
        .flatten()
        .map(|row| row.id)
}

pub async fn get_by_id(
    connection: &mut PoolConnection<Postgres>,
    id: &uuid::Uuid,
) -> Option<(i32, String)> {
    sqlx::query_file!("sql/get/banner/link_banner_by_uuid.sql", id)
        .fetch_optional(connection)
        .await
        .ok()
        .flatten()
        .map(|row| (row.id, row.image_path))
}

pub async fn update_link_states(
    connection: &mut PoolConnection<Postgres>,
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
    .execute(&mut trx)
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
    connection: &mut PoolConnection<Postgres>,
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
    pub last_thirty_days: Option<i64>,
    pub last_seven_days: Option<i64>,
    pub average_weekly: Option<i64>,
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
    connection: &mut PoolConnection<Postgres>,
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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Banner {
    pub link: url::Url,
    pub image: String,
    pub frequency: i16,
    pub is_affiliate: bool,
    pub id: uuid::Uuid,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
    pub filename: String,
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
