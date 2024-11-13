use std::path::PathBuf;

use base64::{engine, Engine};
use chrono::Utc;
use once_cell::sync::Lazy;
use regex::{Regex, RegexSet, RegexSetBuilder};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, PgConnection};

use super::{image, plug::ChargeType};
use crate::{
    charge_price_api::response::condition::TariffPriceResponse,
    slack::{self, LinkPreview, Slack, SlackClient, TextMessage},
};

static REGEX_INTERNAL_TARIFF_NAME: Lazy<regex::Regex> = Lazy::new(|| {
    regex::RegexBuilder::new(r#"[^A-Za-z0-9ß+-_]"#)
        .case_insensitive(true)
        .build()
        .unwrap()
});

#[derive(Clone, Debug, Deserialize)]
pub struct ChargePriceTariff {
    pub id: i32,
    pub relationship_id: uuid::Uuid,
    pub provider_name: String,
    pub provider_id: uuid::Uuid,
    pub slug_name: String,
    pub monthly_fee: f64,
    pub provider_customer_only: bool,
    pub standard: bool,
    pub ad_hoc: bool,
    pub url: Option<String>,
    pub image: Option<i32>,
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct PriceTuple(pub uuid::Uuid, pub uuid::Uuid, pub ChargeType);

impl PartialEq<ChargePriceTariff> for ChargePriceTariff {
    fn eq(&self, other: &ChargePriceTariff) -> bool {
        self.slug_name == other.slug_name
            && self.monthly_fee == other.monthly_fee
            && self.provider_customer_only == other.provider_customer_only
            && self.provider_name == other.provider_name
            && self.ad_hoc == other.ad_hoc
            && self.url == other.url
            && self.provider_id == other.provider_id
            && self.standard == other.standard
    }

    fn ne(&self, other: &ChargePriceTariff) -> bool {
        !self.eq(other)
    }
}

pub static CUSTOMER_ONLY_TARIFFS_NAME: Lazy<RegexSet> = Lazy::new(|| {
    RegexSetBuilder::new(&["privat", "kunde", "business", "bestand", "profi", "plus"])
        .case_insensitive(true)
        .build()
        .unwrap()
});

impl ChargePriceTariff {
    fn is_ad_hoc(&self) -> bool {
        let slug_lower = self.slug_name.to_lowercase();
        let pattern = "ad-hoc";
        self.ad_hoc || slug_lower.ends_with(pattern) || slug_lower.starts_with(pattern)
    }
    pub async fn save(
        &mut self,
        transaction: &mut PgConnection,
        ad_hoc_image: Option<i32>,
    ) -> Result<(i32, Option<String>), sqlx::error::Error> {
        let affiliate_link_str = self.url.as_ref().map(|i| i.to_string());
        let slug_name = self.slug_name.clone();
        self.fix_provider_only_slug_name();
        let (id, internal_name) = match get_by_relation_id(&mut *transaction, &self.relationship_id)
            .await?
        {
            Some(tariff) if self != &tariff => {
                let internal_name = sqlx::query_file_scalar!(
                    "sql/update/tariff/tariff.sql",
                    tariff.id,
                    self.slug_name,
                    self.monthly_fee,
                    affiliate_link_str,
                    self.provider_name,
                    self.provider_customer_only,
                    self.standard,
                    self.ad_hoc,
                    self.provider_id
                )
                .fetch_one(&mut *transaction)
                .await?;
                self.image = tariff.image;
                match tariff.image {
                    None if !tariff.standard && self.standard => (tariff.id, Some(internal_name)),
                    _ => (tariff.id, None),
                }
            }
            Some(tariff) => (tariff.id, None),
            None => {
                let (image_id, internal_name) = if self.is_ad_hoc() {
                    (ad_hoc_image, String::from("lf_spontan"))
                } else {
                    (None, self.normalize_internal_name(&slug_name))
                };

                tracing::debug!(msg = "Insert or update new tariff", self.slug_name, self.provider_name, self.standard, %self.relationship_id, internal_name, image_id);

                let id = sqlx::query_file_scalar!(
                    "sql/insert/tariff.sql",
                    self.relationship_id,
                    self.slug_name,
                    self.monthly_fee,
                    affiliate_link_str,
                    internal_name,
                    image_id,
                    self.provider_name,
                    self.provider_customer_only,
                    self.standard,
                    self.ad_hoc,
                    self.provider_id
                )
                .fetch_one(&mut *transaction)
                .await?;
                self.image = image_id;

                match image_id {
                    None if self.standard => (id, Some(internal_name)),
                    _ => (id, None),
                }
            }
        };
        self.id = id;
        Ok((id, internal_name))
    }

    fn fix_provider_only_slug_name(&mut self) {
        if self.provider_customer_only && !CUSTOMER_ONLY_TARIFFS_NAME.is_match(&self.slug_name) {
            self.slug_name.push_str(" (Kundentarif)");
        }
    }

    async fn send_slack_new_tariff_message(
        &self,
        slack_client: &Option<Slack>,
        cpo_name: &str,
        internal_name: &str,
    ) {
        tracing::info!(
            status = "tariff without an image",
            message = "send new slack message",
            id = self.id,
            tariff_name = self.slug_name,
            internal_name,
            cpo_name,
            relationship_id = self.relationship_id.to_string()
        );
        match slack_client {
            Some(slack) if slack.count() < 6 => {
                let tariff_link = parse_url_from_base64_query(&self.url);
                let link = if let Some(link) = tariff_link {
                    LinkPreview {
                        text: link.host_str().unwrap_or_default(),
                        link: &link,
                    }
                    .to_string()
                } else {
                    String::from("none link")
                };
                let message = format!(
							"Hi {}, I found a new card {:#?} without an image.\nHere are some useful information:\nCPO: {}\nName Internal: {}\nLink: {}",
							slack::MALIK,
							self.slug_name,
							cpo_name,
							internal_name,
							link
						);
                slack
                    .send(TextMessage {
                        emoji: Some(slack::Emoji::New),
                        text: message,
                    })
                    .await;
                slack.inc_count();
            }
            _ => {}
        }
    }

    fn normalize_internal_name(&self, text: &str) -> String {
        let tariff_name = REGEX_INTERNAL_TARIFF_NAME
            .replace_all(text, "")
            .replace("/", "");
        let provider_name = REGEX_INTERNAL_TARIFF_NAME.replace_all(&self.provider_name, "");

        format!("{provider_name}_{tariff_name}").to_lowercase()
    }
}

pub async fn get_filter(connection: &mut PgConnection) -> Result<Vec<Regex>, sqlx::Error> {
    // maybe use regex set
    let filter_list = sqlx::query_file!("sql/get/all_filter.sql")
        .fetch_all(&mut *connection)
        .await?
        .into_iter()
        .filter_map(|row| {
            // maybe try regex set https://docs.rs/regex/latest/regex/struct.RegexSet.html (faster)
            regex::RegexBuilder::new(&row.value)
                .case_insensitive(true)
                .build()
                .ok()
        })
        .collect::<Vec<_>>();

    Ok(filter_list)
}

pub struct TariffContext<'a> {
    pub transaction: &'a mut PgConnection,
    pub response: &'a TariffPriceResponse,
    pub slack: &'a Option<Slack>,
}

pub async fn save_tariffs(context: TariffContext<'_>) -> Result<(), sqlx::Error> {
    let filter_list = get_filter(context.transaction).await?;
    context.slack.reset_count(); // TODO slack !?

    for tariff_response in &context.response.tariffs {
        let mut tariff = tariff_response.into_tariff(&filter_list);
        let image_ad_hoc = image::get_ad_hoc(&mut *context.transaction).await;
        let (_, internal_tariff_name) =
            tariff.save(&mut *context.transaction, image_ad_hoc).await?;

        if let (Some(internal_name), false) = (internal_tariff_name, tariff.is_ad_hoc()) {
            tariff
                .send_slack_new_tariff_message(
                    context.slack,
                    &tariff_response.operator.name,
                    &internal_name,
                )
                .await;
        }
    }
    Ok(())
}

pub async fn get_by_relation_id(
    transaction: &mut PgConnection,
    relation_id: &uuid::Uuid,
) -> Result<Option<ChargePriceTariff>, sqlx::error::Error> {
    let row = sqlx::query_file_as!(
        ChargePriceTariff,
        "sql/get/tariff/tariff_by_relationship_id.sql",
        relation_id
    )
    .fetch_optional(transaction)
    .await?;
    Ok(row)
}
pub async fn get_by_public_id(
    connection: &mut PgConnection,
    pub_tariff_id: &uuid::Uuid,
) -> Result<Option<ChargePriceTariff>, sqlx::error::Error> {
    let row = sqlx::query_file_as!(
        ChargePriceTariff,
        "sql/get/tariff/tariff_by_public_id.sql",
        pub_tariff_id
    )
    .fetch_optional(connection)
    .await?;
    Ok(row)
}
pub async fn get_by_name(
    connection: &mut PgConnection,
    name: &str,
) -> Result<Vec<i32>, sqlx::error::Error> {
    sqlx::query_file_scalar!("sql/get/tariff/tariff_by_internal_name.sql", name)
        .fetch_all(connection)
        .await
}

pub async fn get_count(connection: &mut PgConnection) -> Result<i64, sqlx::error::Error> {
    let count = sqlx::query_file_scalar!("sql/get/tariff/tariff_count.sql")
        .fetch_one(connection)
        .await?;

    Ok(count.unwrap_or_default())
}

pub fn parse_url_from_base64_query(link: &Option<String>) -> Option<url::Url> {
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
        .and_then(|(_, value)| url::Url::parse(&value).ok())
}

pub mod admin {
    use super::*;

    #[derive(Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TariffIntern {
        pub relationship_id: uuid::Uuid,
        pub id: i32,
        pub slug_name: String,
        pub url: Option<String>,
        pub updated: chrono::DateTime<Utc>,
        pub provider_name: String,
        pub internal_name: String,
        pub image: Option<ImageIntern>,
        pub standard: bool,
        pub override_standard: bool,
        pub notes: String,
        pub provider_customer_only: bool,
        pub hide: bool,
        pub monthly_fee: f64,
        pub image_id: Option<i32>,
    }

    #[derive(Clone, Serialize)]
    pub struct ImageIntern {
        pub filename: Option<String>,
        pub checksum: String,
    }

    #[derive(Debug, Clone, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UpdateTariffInternal {
        id: i32,
        internal_name: String,
        notes: String,
        hide: bool,
        override_standard: bool,
        url: Option<Url>,
        image_id: Option<i32>,
    }

    pub async fn set_image(
        transaction: &mut PgConnection,
        tariff_id: i32,
        image_id: Option<i32>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query_file!("sql/update/tariff/image_tariff_id.sql", image_id, tariff_id)
            .execute(transaction)
            .await?;
        Ok(())
    }

    pub async fn update_partial(
        connection: &mut PgConnection,
        tariff: &UpdateTariffInternal,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = connection.begin().await?;
        sqlx::query_file!(
            "sql/update/tariff/tariff_internal_partial.sql",
            tariff.id,
            tariff.notes,
            tariff.override_standard,
            tariff.internal_name,
            tariff.hide,
            tariff.url.as_ref().map(|u| u.as_str())
        )
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;

        if let Some(image_id) = tariff.image_id {
            if let Err(error) =
                image::update_image_file_name(connection, &tariff.internal_name, image_id, None)
                    .await
            {
                tracing::error!(
                    tariff_id = tariff.id,
                    internal_name = tariff.internal_name,
                    image_id = image_id,
                    %error,
                    "Could update internal tariff name",
                )
            }
        }

        Ok(())
    }

    pub async fn set_internal_name(
        connection: &mut PgConnection,
        tariff_id: i32,
        name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query_file!(
            "sql/update/tariff/tariff_internal_name.sql",
            name,
            tariff_id
        )
        .execute(connection)
        .await?;

        Ok(())
    }
    pub async fn get_all(
        connection: &mut PgConnection,
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
                    relationship_id: row.relationship_id,
                    id: row.id,
                    slug_name: row.slug_name.clone(),
                    url: parse_url_from_base64_query(&row.url).map(|value| value.to_string()),
                    image: image,
                    notes: row.note.clone(),
                    internal_name: row.internal_name.clone(),
                    provider_name: row.provider_name.clone(),
                    standard: row.standard,
                    updated: row.updated,
                    override_standard: row.override_standard,
                    provider_customer_only: row.provider_customer_only,
                    hide: row.hide,
                    image_id: row.image_id,
                    monthly_fee: row.monthly_fee,
                }
            })
            .collect();
        Ok(rows)
    }
}

pub mod v3 {
    use crate::api::tariff::v3;

    use super::*;

    pub async fn get_tariffs(
        connection: &mut PgConnection,
        domain: &url::Url,
        only_standard: bool,
    ) -> Result<Vec<v3::Tariff>, sqlx::Error> {
        if only_standard {
            sqlx::query_file_as!(
                v3::Tariff,
                "sql/get/tariff/v3/tariff_only_standard.sql",
                domain.to_string(),
            )
            .fetch_all(connection)
            .await
        } else {
            sqlx::query_file_as!(
                v3::Tariff,
                "sql/get/tariff/v3/tariff_all.sql",
                domain.to_string(),
            )
            .fetch_all(connection)
            .await
        }
    }

    pub async fn get_standard_and_custom_with_operators(
        connection: &mut PgConnection,
        domain: &url::Url,
        add: &[uuid::Uuid],
        remove: &[uuid::Uuid],
        operator_ids: &[uuid::Uuid],
    ) -> Result<Vec<v3::Tariff>, sqlx::Error> {
        sqlx::query_file_as!(
            v3::Tariff,
            "sql/get/tariff/v3/tariff_custom_operators.sql",
            domain.to_string(),
            add,
            remove,
            operator_ids
        )
        .fetch_all(connection)
        .await
    }

    pub async fn get_all_for_operators(
        connection: &mut PgConnection,
        domain: &url::Url,
        operator_ids: &[uuid::Uuid],
    ) -> Result<Vec<v3::Tariff>, sqlx::Error> {
        sqlx::query_file_as!(
            v3::Tariff,
            "sql/get/tariff/v3/tariff_all_with_operators.sql",
            domain.to_string(),
            &operator_ids
        )
        .fetch_all(connection)
        .await
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
