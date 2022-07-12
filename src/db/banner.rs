use ::chrono::serde::ts_seconds;
use chrono::Utc;
use serde::Serialize;
use sqlx::{pool::PoolConnection, Acquire, Postgres};
use urlencoding::encode;

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
            let is_affiliate = row.is_affiliate;
            let url_str = if is_affiliate {
                format!("{}affiliate?url={}", api_url, encode(&row.source))
            } else {
                row.source.to_owned()
            };

            let banner_url = format!("{}img/banners/{}", api_url, row.image);
            match url::Url::parse(&url_str) {
                Ok(link) => Some(Banner {
                    id: row.id,
                    link,
                    image: banner_url,
                    is_affiliate: is_affiliate,
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
    sqlx::query_file!("sql/get/single_link.sql", link.as_str())
        .fetch_optional(connection)
        .await
        .ok()
        .flatten()
        .map(|row| row.id)
}

pub async fn update_link_states(
    connection: &mut PoolConnection<Postgres>,
    link_id: i32,
    plattform: &PlattformType,
) -> Result<(), sqlx::Error> {
    let mut trx = connection.begin().await?;
    sqlx::query_file!("sql/insert_update/link_states.sql", link_id, plattform as _)
        .execute(&mut trx)
        .await?;
    trx.commit().await?;
    Ok(())
}

#[derive(Serialize, Debug)]
pub struct Banner {
    pub link: url::Url,
    pub image: String,
    pub high_priority: bool,
    pub is_affiliate: bool,
    pub id: uuid::Uuid,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
}

// impl From<Banner> for BannerJson {
//     fn from(banner: Banner) -> Self {
//         Self {
//             link: url::Url::parse(&format!("http://example.com?ref={}", banner.link)).unwrap(),
//             image_path: banner.image,
//             frequency: banner.frequency,
//             id: banner.id,
//             updated: banner.updated,
//         }
//     }
// }

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
