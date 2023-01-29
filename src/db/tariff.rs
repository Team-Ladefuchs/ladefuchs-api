use std::path::PathBuf;

use base64::{engine, Engine};
use chrono::Utc;
use once_cell::sync::Lazy;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, Postgres, Transaction};

use super::{image, plug::ChargeType};
use crate::slack::{self, Slack, SlackClient};

static REGEX_INTERNAL_TARIFF_NAME: Lazy<regex::Regex> = Lazy::new(|| {
    regex::RegexBuilder::new(r#"[^A-Za-z0-9ß+-_]"#)
        .case_insensitive(true)
        .build()
        .unwrap()
});

#[derive(Clone, Debug, Deserialize)]
pub struct Tariff {
    pub id: i32,
    pub relationship_id: uuid::Uuid,
    pub msp_id: i32,
    pub slug_name: String,
    pub monthly_fee: f64,
    pub url: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct TariffIntern {
    pub id: uuid::Uuid,
    pub slug_name: String,
    pub url: Option<String>,
    pub updated: chrono::DateTime<Utc>,
    pub msp_name: String,
    pub internal_name: String,
    pub image: Option<ImageIntern>,
    pub visible: bool,
}

#[derive(Clone, Serialize)]
pub struct ImageIntern {
    pub filename: Option<String>,
    pub checksum: String,
}

impl Tariff {
    pub async fn save(
        &self,
        transaction: &mut sqlx::Transaction<'_, Postgres>,
        cpo_name: &str,
        slack_client: &Option<Slack>,
    ) -> Result<i32, sqlx::error::Error> {
        let affilate_link_str = self.url.as_ref().map(|i| i.to_string());

        let tariff_id = match get_by_id(&mut *transaction, &self.relationship_id).await? {
            Some(tariff)
                if self.slug_name != tariff.slug_name
                    || self.monthly_fee != tariff.monthly_fee
                    || self.url != tariff.url =>
            {
                sqlx::query_file!(
                    "sql/update/tariff.sql",
                    tariff.id,
                    self.slug_name,
                    self.monthly_fee,
                    affilate_link_str
                )
                .execute(&mut *transaction)
                .await?;
                tariff.id
            }
            Some(tariff) => tariff.id,
            None => {
                let (image_id, internal_name) = if self.slug_name.eq_ignore_ascii_case("ad-hoc") {
                    (
                        image::get_ad_hoc(&mut *transaction).await,
                        String::from("lf_spontan"),
                    )
                } else {
                    (None, self.normalize_internal_name(&self.slug_name))
                };

                tracing::info!(
                    msg = "Inserting new tariff",
                    tariff_name = self.slug_name,
                    internal_name,
                    msp_id = self.msp_id
                );

                if matches!(slack_client, Some(slack) if image_id.is_none() && slack.count() < 5) {
                    self.send_slack_message(slack_client, cpo_name, &internal_name)
                        .await;
                }

                let id = sqlx::query_file_scalar!(
                    "sql/insert/tariff.sql",
                    self.msp_id,
                    self.relationship_id,
                    self.slug_name,
                    self.monthly_fee,
                    affilate_link_str,
                    internal_name,
                    image_id
                )
                .fetch_one(&mut *transaction)
                .await?;
                id
            }
        };
        Ok(tariff_id)
    }

    async fn send_slack_message(
        &self,
        slack_client: &Option<Slack>,
        cpo_name: &str,
        internal_name: &str,
    ) {
        let tariff_link = parse_url_from_base64_query(&self.url);
        let link = if let Some(url) = tariff_link {
            format!("<{}>", urlencoding::decode(&url).unwrap_or_default())
        } else {
            String::from("none link")
        };

        let message = format!(
						"Hi {}, I found a new card {:#?} without an image.\nHere are some useful information:\nCPO: {}\nName Internal: {}\n{}",
						slack::MALIK,
						self.slug_name,
						cpo_name,
						internal_name,
						link
					);
        slack_client.send(Some(slack::Emoji::New), &message).await;
        slack_client.inc_count();
    }

    fn normalize_internal_name(&self, text: &str) -> String {
        REGEX_INTERNAL_TARIFF_NAME
            .replace_all(text, "")
            .to_lowercase()
    }
}

pub async fn get_by_id(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    relation_id: &uuid::Uuid,
) -> Result<Option<Tariff>, sqlx::error::Error> {
    let row = sqlx::query_file_as!(Tariff, "sql/get/tariff/tariff_by_id.sql", relation_id)
        .fetch_optional(transaction)
        .await?;
    Ok(row)
}

pub async fn get_by_name(
    connection: &mut PoolConnection<Postgres>,
    name: &str,
) -> Result<i32, sqlx::error::Error> {
    let tariff_id = sqlx::query_file_scalar!("sql/get/tariff/tariff_by_internal_name.sql", name)
        .fetch_one(connection)
        .await?;
    Ok(tariff_id)
}

pub async fn get_all_intern(
    connection: &mut PoolConnection<Postgres>,
) -> Result<Vec<TariffIntern>, sqlx::error::Error> {
    let rows = sqlx::query_file!("sql/get/tariff/tariffs_intern.sql")
        .fetch_all(connection)
        .await?
        .iter()
        .map(|row| {
            let image = row.checksum.as_ref().map(|checksum| ImageIntern {
                filename: row.file_path.as_ref().map(|p| {
                    PathBuf::from(p)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                }),

                checksum: checksum.to_string(),
            });
            TariffIntern {
                id: row.id,
                slug_name: row.slug_name.clone(),
                url: parse_url_from_base64_query(&row.url),
                image: image,
                internal_name: row.internal_name.clone(),
                msp_name: row.msp_name.clone(),
                visible: row.visible,
                updated: row.updated,
            }
        })
        .collect();
    Ok(rows)
}

pub fn parse_url_from_base64_query(link: &Option<String>) -> Option<String> {
    let link = link.as_ref()?;

    let mut url = Url::parse(link.as_str()).ok()?;

    let tokens = url
        .query_pairs()
        .find(|(key, _)| key == "token")
        .and_then(|(_, value)| {
            engine::general_purpose::STANDARD
                .decode(value.as_bytes())
                .ok()
        })
        .and_then(|vec| String::from_utf8(vec).ok())?;

    url.set_query(Some(&tokens));

    url.query_pairs()
        .find(|(key, _)| key == "url")
        .map(|(_, value)| value.to_string())
}

#[derive(Serialize, Debug, Clone)]
pub struct TariffsWithBlockingFee {
    pub relationship_id: uuid::Uuid,
    pub tariff_id: i32,
    pub cpo_id: i32,
    pub cpo_network: uuid::Uuid,
    pub cpo_name: String,
}

pub async fn get_all_blocking_fee(
    connection: &mut PoolConnection<Postgres>,
) -> Result<Vec<TariffsWithBlockingFee>, sqlx::error::Error> {
    sqlx::query_file_as!(
        TariffsWithBlockingFee,
        "sql/get/tariff/tariffs_with_blocking_fee.sql"
    )
    .fetch_all(connection)
    .await
}

#[derive(Clone, Debug)]
pub struct TariffBlockingPrice {
    pub tariff_id: i32,
    pub cpo_id: i32,
    pub price: f64,
    pub plug: ChargeType,
}

pub async fn set_image(
    transaction: &mut Transaction<'_, Postgres>,
    tariff_id: i32,
    image_id: Option<i32>,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/update/image_tariff_id.sql", image_id, tariff_id)
        .execute(transaction)
        .await?;
    Ok(())
}

pub async fn set_internal_name(
    transaction: &mut Transaction<'_, Postgres>,
    tariff_id: i32,
    name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/update/tariff_internal_name.sql", name, tariff_id)
        .execute(transaction)
        .await?;

    Ok(())
}

impl TariffBlockingPrice {
    pub async fn save(
        &self,
        transaction: &mut sqlx::Transaction<'_, Postgres>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query_file!(
            "sql/update/tariff_update_blocking_price.sql",
            self.price,
            self.cpo_id,
            self.tariff_id,
            self.plug as _
        )
        .execute(transaction)
        .await?;
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::str::FromStr;

//     use super::*;
//     use crate::{config, db::connect};

//     #[tokio::test]
//     async fn test_get_cpo() {
//         let config = config::read_config().unwrap();
//         let pool = connect(&config.database_url).await.unwrap();
//         let mut conn = pool.acquire().await.unwrap();
//         let tarif = Tarif {
//             relationship_id: uuid::Uuid::from_str("0e21478b-b829-45c1-80b8-4b0aee473269").unwrap(),
//             msp_id: 1,
//             vehicle_id: 1,
//             slug_name: "test tarif1".into(),
//             monthly_fee: 10.0,
//         };
//         let id = tarif.save(&mut conn).await.unwrap();
//         let tarif2 = Tarif {
//             slug_name: "test tarif neu".into(),
//             ..tarif
//         };
//         let id2 = tarif2.save(&mut conn).await.unwrap();
//         assert_eq!(id, id2);
//     }
// }
