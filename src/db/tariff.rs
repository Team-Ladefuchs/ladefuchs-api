use sqlx::{pool::PoolConnection, Postgres};

pub struct Tariff<'a> {
    pub relationship_id: uuid::Uuid,
    pub msp_id: i32,
    pub slug_name: String,
    pub monthly_fee: f64,
    pub url: &'a Option<url::Url>,
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
