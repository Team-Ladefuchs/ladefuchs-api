use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use base64::{engine, Engine};
use chrono::Utc;
use once_cell::sync::Lazy;
use percent_encoding::percent_decode_str;
use regex::Regex;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Connection, PgConnection};

use super::{charge_price::ChargePrice, image, plug::ChargeType};
use crate::{
    charge_price_api::response::ApiResponse,
    slack::{self, Slack, SlackClient},
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
    pub slug_name: String,
    pub monthly_fee: f64,
    pub provider_customer_only: bool,
    pub standard: bool,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]

pub struct TariffV1 {
    pub identifier: uuid::Uuid,
    pub provider: Provider,
    pub name: String,
    pub monthly_fee: f64,
    pub note: String,
    pub image: Option<String>,
    pub standard: bool,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]

pub struct Provider {
    pub identifier: uuid::Uuid,
    pub name: String,
    pub customer_only: bool,
}

impl From<Value> for Provider {
    fn from(value: Value) -> Self {
        serde_json::from_value(value).unwrap()
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TariffAdminIntern {
    pub relationship_id: uuid::Uuid,
    pub id: i32,
    pub slug_name: String,
    pub url: Option<String>,
    pub updated: chrono::DateTime<Utc>,
    pub msp_name: String,
    pub internal_name: String,
    pub image: Option<ImageIntern>,
    pub standard: bool,
    pub notes: String,
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
    is_enabled: bool,
}

#[derive(Clone, Debug)]
pub struct TariffBlockingPrice {
    pub tariff_relation: uuid::Uuid,
    pub operator_network: uuid::Uuid,
    pub blocking_fee: f64,
    pub plug: ChargeType,
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct PriceTuple(pub uuid::Uuid, pub uuid::Uuid, pub ChargeType);

impl PartialEq<ChargePriceTariff> for ChargePriceTariff {
    fn eq(&self, other: &ChargePriceTariff) -> bool {
        self.slug_name == other.slug_name
            && self.monthly_fee == other.monthly_fee
            && self.provider_customer_only == other.provider_customer_only
            && self.provider_name == other.provider_name
            && self.url == other.url
            && self.standard == other.standard
    }

    fn ne(&self, other: &ChargePriceTariff) -> bool {
        !self.eq(other)
    }
}

impl ChargePriceTariff {
    pub async fn save(
        &mut self,
        transaction: &mut PgConnection,
        operator_name: &str,
        slack_client: &Option<Slack>,
    ) -> Result<(), sqlx::error::Error> {
        let affiliate_link_str = self.url.as_ref().map(|i| i.to_string());

        self.id = match get_by_id(&mut *transaction, &self.relationship_id).await? {
            Some(tariff) if self != &tariff => {
                sqlx::query_file!(
                    "sql/update/tariff/tariff.sql",
                    tariff.id,
                    self.slug_name.trim(),
                    self.monthly_fee,
                    affiliate_link_str,
                    self.provider_name,
                    self.provider_customer_only,
                    self.standard,
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

                tracing::debug!(
                    msg = "Insert or update new tariff",
                    tariff_name = self.slug_name,
                    internal_name,
                    provider_name = self.provider_name
                );

                // only send if tariffs is standard (monthly=0, provider_customer_only=false, standard=?)
                if matches!(slack_client, Some(slack) if self.standard && image_id.is_none() && slack.count() < 5)
                {
                    tracing::info!(
                        status = "new tariff",
                        message = "send new slack message",
                        tariff_name = self.slug_name,
                        relationship_id = self.relationship_id.to_string()
                    );

                    self.send_slack_new_tariff_message(
                        slack_client,
                        &operator_name,
                        &internal_name,
                    )
                    .await;
                }

                sqlx::query_file_scalar!(
                    "sql/insert/tariff.sql",
                    self.relationship_id,
                    self.slug_name.trim(),
                    self.monthly_fee,
                    affiliate_link_str,
                    internal_name,
                    image_id,
                    self.provider_name,
                    self.provider_customer_only,
                    self.standard
                )
                .fetch_one(&mut *transaction)
                .await?
            }
        };

        Ok(())
    }

    async fn send_slack_new_tariff_message(
        &self,
        slack_client: &Option<Slack>,
        cpo_name: &str,
        internal_name: &str,
    ) {
        let tariff_link = parse_url_from_base64_query(&self.url);
        let link = if let Some(url) = tariff_link {
            format!(
                "<{}>",
                percent_decode_str(&url).decode_utf8().unwrap_or_default()
            )
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
    pub responses: &'a [ApiResponse],
    pub standard_operators: HashSet<uuid::Uuid>,
    pub slack: &'a Option<Slack>,
}

pub async fn save_tariffs(context: TariffContext<'_>) -> Result<Vec<ChargePrice>, sqlx::Error> {
    let filter_list = get_filter(context.transaction).await?;
    context.slack.reset_count(); // TODO slack !?
    let mut tariffs: HashMap<uuid::Uuid, (ChargePriceTariff, &str)> = HashMap::new();
    let mut prices = Vec::with_capacity(context.responses.len());
    for api_response in context.responses {
        for provider in &api_response.providers {
            let is_standard_operator = context
                .standard_operators
                .contains(&api_response.operator.network);
            let tariff = provider.into_tariff(
                provider.attributes.provider.clone(),
                &filter_list,
                is_standard_operator,
            );

            if let Some((item, _)) = tariffs.get_mut(&tariff.relationship_id) {
                if !item.standard && tariff.standard {
                    item.standard = tariff.standard;
                } else if item.standard && !tariff.standard && is_standard_operator {
                    item.standard = tariff.standard;
                }
            } else {
                tariffs.insert(
                    tariff.relationship_id,
                    (tariff.clone(), &api_response.operator.slug_name),
                );
            }
        }
    }

    for (tariff, operator_name) in tariffs.values_mut() {
        tariff
            .save(context.transaction, operator_name, context.slack)
            .await?;
    }

    for api_response in context.responses {
        for provider in &api_response.providers {
            let tariff = tariffs.get(&provider.relationship_id());
            for price in &provider.attributes.charge_point_prices {
                tracing::debug!(provider=%provider.attributes.provider, price=%price.price, tariff=%provider.attributes.tariff_name, plug=%price.plug);
                let plug = &price.plug;

                if let Some((tariff, _)) = &tariff {
                    prices.push(ChargePrice {
                        operator_id: api_response.operator.id,
                        operator_network: api_response.operator.network,
                        tariff_relation: tariff.relationship_id,
                        tariff_id: tariff.id,
                        c_type: plug.into(),
                        price: price.price,
                        blocking_fee: 0.0,
                        blocking_fee_start: price.blocking_fee_start.unwrap_or_default(),
                    });
                }
            }
        }
    }

    Ok(prices)
}

pub async fn get_by_id(
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

pub async fn get_by_name(
    connection: &mut PgConnection,
    name: &str,
) -> Result<i32, sqlx::error::Error> {
    let tariff_id = sqlx::query_file_scalar!("sql/get/tariff/tariff_by_internal_name.sql", name)
        .fetch_one(connection)
        .await?;
    Ok(tariff_id)
}

pub async fn get_all_intern(
    connection: &mut PgConnection,
) -> Result<Vec<TariffAdminIntern>, sqlx::error::Error> {
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
            TariffAdminIntern {
                relationship_id: row.relationship_id,
                id: row.id,
                slug_name: row.slug_name.clone(),
                url: parse_url_from_base64_query(&row.url),
                image: image,
                notes: row.note.clone(),
                internal_name: row.internal_name.clone(),
                msp_name: row.msp_name.clone(),
                standard: row.standard,
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
    pub operator_network: uuid::Uuid,
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
        tariff.is_enabled,
        tariff.internal_name
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
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

pub async fn get_tariffs_v1(
    connection: &mut PgConnection,
    domain: &url::Url,
    only_standard: bool,
) -> Result<Vec<TariffV1>, sqlx::Error> {
    if only_standard {
        sqlx::query_file_as!(
            TariffV1,
            "sql/get/tariff/tariff_only_standard_v1.sql",
            domain.to_string(),
        )
        .fetch_all(connection)
        .await
    } else {
        sqlx::query_file_as!(
            TariffV1,
            "sql/get/tariff/tariff_all_v1.sql",
            domain.to_string(),
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
