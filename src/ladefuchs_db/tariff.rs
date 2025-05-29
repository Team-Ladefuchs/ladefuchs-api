use std::path::PathBuf;

use admin::UpdateTariffInternal;
use base64::{Engine, engine};
use chrono::Utc;
use once_cell::sync::Lazy;
use regex::{RegexSet, RegexSetBuilder};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, PgConnection};

use super::{image, plug::ChargeType};
use crate::eco_movement::db::tariff::EcoTariff;

static REGEX_INTERNAL_TARIFF_NAME: Lazy<regex::Regex> = Lazy::new(|| {
    regex::RegexBuilder::new(r#"[^A-Za-z0-9ß+-_]"#)
        .case_insensitive(true)
        .build()
        .unwrap()
});

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub struct Tariff {
    pub id: i32,
    pub relationship_id: uuid::Uuid,
    pub provider_name: String,
    pub provider_id: uuid::Uuid,
    pub slug_name: String,
    pub monthly_fee: f64,
    pub provider_customer_only: bool,
    pub standard: bool,
    pub ad_hoc: bool,
    pub brand_only: bool,
    pub url: Option<String>,
    pub image: Option<i32>,
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct PriceTuple(pub uuid::Uuid, pub uuid::Uuid, pub ChargeType);

pub static CUSTOMER_ONLY_TARIFFS_NAME: Lazy<RegexSet> = Lazy::new(|| {
    RegexSetBuilder::new(&[
        "customer",
        "private",
        "kunde",
        "subscribers",
        "business",
        "bestand",
        "profi",
        "audi",
        "mercedes",
        "bmw",
        "seat",
        "fleet",
    ])
    .case_insensitive(true)
    .build()
    .unwrap()
});

pub async fn add_or_update_tariffs(
    connection: &mut PgConnection,
    tariffs: &[EcoTariff],
) -> Result<(), sqlx::error::Error> {
    let ad_hoc_image = image::get_ad_hoc(connection).await;
    for tariff in tariffs {
        if let Err(err) = add_or_update_tariff(connection, tariff, ad_hoc_image).await {
            tracing::error!(error = %err.to_string(), ?tariff, "Could update or insert");
            return Err(err);
        }
        // let (image_id, internal_tariff_name) =
        // if let (Some(internal_name), false) = (internal_tariff_name, tariff.is_ad_hoc()) {
        //     tariff
        //         .send_slack_new_tariff_message(
        //             context.slack,
        //             &tariff_response.operator.name,
        //             &internal_name,
        //         )
        //         .await;
        // }
    }
    Ok(())
}

pub async fn add_or_update_tariff(
    connection: &mut PgConnection,
    tariff: &EcoTariff,
    ad_hoc_image: Option<i32>,
) -> Result<(i32, Option<String>), sqlx::error::Error> {
    let (tariff_name, internal_name) = if tariff.is_ad_hoc() {
        (String::from("Ad-hoc"), String::from("lf_spontan"))
    } else {
        (
            tariff.name.clone(),
            normalize_internal_name(&tariff.name, &tariff.provider_name),
        )
    };

    let ret = match get_by_internal_name_and_provider_or_network(
        connection,
        &internal_name,
        &tariff.provider_name,
        &tariff.network,
    )
    .await?
    {
        Some(current_tariff) => {
            tracing::debug!(new = ?tariff, current=?current_tariff, "update tariff");
            sqlx::query_file!(
                "sql/update/tariff/tariff.sql",
                current_tariff.id,
                tariff_name,
                tariff.subscription_fee,
                tariff.provider_name,
                tariff.is_customer_only(),
                tariff.is_standard(),
                tariff.is_ad_hoc(),
                tariff.network
            )
            .execute(&mut *connection)
            .await?;

            match current_tariff.image {
                None if !current_tariff.standard && tariff.is_standard() => {
                    (current_tariff.id, Some(internal_name))
                }
                _ => (current_tariff.id, None),
            }
        }
        None => {
            let image_id = tariff.is_ad_hoc().then_some(ad_hoc_image).flatten();

            tracing::debug!(?tariff, "Insert tariff");
            let website: Option<String> = None;
            let id = sqlx::query_file_scalar!(
                "sql/insert/tariff.sql",
                tariff.network,
                tariff_name,
                tariff.subscription_fee,
                website,
                internal_name,
                image_id,
                tariff.provider_name,
                tariff.is_customer_only(),
                tariff.is_standard(),
                tariff.is_ad_hoc(),
                uuid::Uuid::new_v4()
            )
            .fetch_one(&mut *connection)
            .await?;

            match image_id {
                None if tariff.is_standard() => (id, Some(internal_name)),
                _ => (id, None),
            }
        }
    };

    Ok(ret)
}

pub async fn get_by_internal_name_and_provider_or_network(
    connection: &mut PgConnection,
    internal_name: &str,
    provider_name: &str,
    external_id: &uuid::Uuid,
) -> Result<Option<Tariff>, sqlx::error::Error> {
    sqlx::query_file_as!(
        Tariff,
        "sql/get/tariff/tariff_by_internal_name_and_provider.sql",
        internal_name,
        provider_name,
        external_id
    )
    .fetch_optional(connection)
    .await
}

fn normalize_internal_name(tariff: &str, provider_name: &str) -> String {
    let tariff_name = REGEX_INTERNAL_TARIFF_NAME
        .replace_all(tariff, "")
        .replace("/", "");
    let provider_name = REGEX_INTERNAL_TARIFF_NAME.replace_all(provider_name, "");

    format!("{provider_name}_{tariff_name}").to_lowercase()
}

pub async fn get_by_public_id(
    connection: &mut PgConnection,
    pub_tariff_id: &uuid::Uuid,
) -> Result<Option<Tariff>, sqlx::error::Error> {
    let row = sqlx::query_file_as!(
        Tariff,
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

// pub async fn get_count(connection: &mut PgConnection) -> Result<i64, sqlx::error::Error> {
//     let count = sqlx::query_file_scalar!("sql/get/tariff/tariff_count.sql")
//         .fetch_one(connection)
//         .await?;

//     Ok(count.unwrap_or_default())
// }

pub fn is_cp_aff_link(link: &url::Url) -> bool {
    link.domain() != Some("api.chargeprice.app")
}

pub fn parse_url_from_base64_query(link: &url::Url) -> Option<url::Url> {
    let mut url = link.clone();
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
        .map(|mut parsed_url| {
            parsed_url.set_query(None);
            parsed_url
        })
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
        pub id: i32,
        pub internal_name: String,
        pub notes: String,
        pub hide: bool,
        pub url: Option<Url>,
        pub image_id: Option<i32>,
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
        sqlx::query_file!(
            "sql/update/tariff/tariff_internal_partial.sql",
            tariff.id,
            tariff.notes,
            tariff.internal_name,
            tariff.hide,
            tariff.url.as_ref().map(|u| u.as_str())
        )
        .execute(&mut *connection)
        .await?;

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
                    url: row.url.clone(),
                    image,
                    notes: row.note.clone(),
                    internal_name: row.internal_name.clone(),
                    provider_name: row.provider_name.clone(),
                    standard: row.standard,
                    updated: row.updated,
                    override_standard: false,
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
