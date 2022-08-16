use ::chrono::serde::ts_seconds;
use chrono::Utc;
use serde::Serialize;
use sqlx::{pool::PoolConnection, Acquire, Postgres};
use urlencoding::encode;

pub const BANNER_ROUTE: &str = "img/banner";

pub async fn get_all_banner(
    connection: &mut PoolConnection<Postgres>,
    api_url: &url::Url,
) -> Result<Vec<Banner>, sqlx::Error> {
    // let mut transaction = connection.begin().await?;
    let rows = sqlx::query_file!("sql/get/link_banner.sql")
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
                    high_priority: row.high_priority,
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
    sqlx::query_file!("sql/get/link_banner_by_uuid.sql", id)
        .fetch_optional(connection)
        .await
        .ok()
        .flatten()
        .map(|row| (row.id, row.image_path))
}

pub async fn update_link_states(
    connection: &mut PoolConnection<Postgres>,
    link_id: i32,
    plattform: &PlattformType,
    banner_id: Option<i32>,
) -> Result<(), sqlx::Error> {
    let mut trx = connection.begin().await?;
    sqlx::query_file!(
        "sql/insert_update/link_states.sql",
        link_id,
        plattform as _,
        banner_id
    )
    .execute(&mut trx)
    .await?;
    trx.commit().await?;
    Ok(())
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Banner {
    pub link: url::Url,
    pub image: String,
    pub high_priority: bool,
    pub is_affiliate: bool,
    pub id: uuid::Uuid,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
    pub filename: String,
}

#[derive(sqlx::Type, Debug, Clone, Serialize)]
pub enum PlattformType {
    IOS,
    Android,
    Web,
}

impl From<&str> for PlattformType {
    fn from(user_agent: &str) -> Self {
        if user_agent.contains("iPhone") {
            return Self::IOS;
        } else if user_agent.contains("Android") {
            return Self::Android;
        }
        Self::Web
    }
}
