use std::path::PathBuf;

use ::chrono::serde::ts_seconds;
use chrono::Utc;
use sqlx::{pool::PoolConnection, Postgres};

pub struct Tariff<'a> {
    pub relationship_id: uuid::Uuid,
    pub msp_id: i32,
    pub slug_name: String,
    pub monthly_fee: f64,
    pub url: &'a Option<url::Url>,
}

#[derive(Clone, serde::Serialize)]
pub struct TariffIntern {
    pub id: uuid::Uuid,
    pub slug_name: String,
    pub url: Option<String>,
    pub msp_name: String,
    pub image: Option<ImageIntern>,
}

#[derive(Clone, serde::Serialize)]
pub struct ImageIntern {
    pub filename: Option<String>,
    #[serde(with = "ts_seconds")]
    pub updated: chrono::DateTime<Utc>,
    pub checksum: String,
}

impl Tariff<'_> {
    pub async fn save(
        &self,
        transaction: &mut sqlx::Transaction<'_, Postgres>,
    ) -> Result<i32, sqlx::error::Error> {
        match get_by_id(&mut *transaction, &self.relationship_id).await? {
            Some(tarif_id) => {
                sqlx::query_file!(
                    "sql/update/tariff.sql",
                    tarif_id,
                    self.slug_name,
                    self.monthly_fee,
                    self.url.as_ref().map(|i| i.to_string())
                )
                .execute(&mut *transaction)
                .await?;
                Ok(tarif_id)
            }
            None => {
                let id = sqlx::query_file_scalar!(
                    "sql/insert_update/tariff.sql",
                    self.msp_id,
                    self.relationship_id,
                    self.slug_name,
                    self.monthly_fee,
                    self.url.as_ref().map(|i| i.to_string())
                )
                .fetch_one(&mut *transaction)
                .await?;
                Ok(id)
            }
        }
    }
}

pub async fn get_by_id(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    relation_id: &uuid::Uuid,
) -> Result<Option<i32>, sqlx::error::Error> {
    let row = sqlx::query_file!("sql/get/tariff_by_id.sql", relation_id)
        .fetch_optional(transaction)
        .await?;
    Ok(row.map(|r| r.id))
}

pub async fn get_by_name(
    connection: &mut PoolConnection<Postgres>,
    name: &str,
) -> Result<i32, sqlx::error::Error> {
    let tariff_id = sqlx::query_file_scalar!("sql/get/tariff_by_internal_name.sql", name)
        .fetch_one(connection)
        .await?;
    Ok(tariff_id)
}

pub async fn get_all_intern(
    connection: &mut PoolConnection<Postgres>,
) -> Result<Vec<TariffIntern>, sqlx::error::Error> {
    let rows = sqlx::query_file!("sql/get/tarifs_intern.sql")
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
                updated: row.updated.unwrap(),
                checksum: checksum.to_string(),
            });
            TariffIntern {
                id: row.id,
                slug_name: row.slug_name.clone(),
                url: row.url.clone(),
                image: image,
                msp_name: row.msp_name.clone(),
            }
        })
        .collect();
    Ok(rows)
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
