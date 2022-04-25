use sqlx::Postgres;

pub struct Tarif {
    pub relationship_id: uuid::Uuid,
    pub msp_id: i32,
    pub slug_name: String,
    pub monthly_fee: f64,
}

impl Tarif {
    pub async fn save(
        &self,
        transaction: &mut sqlx::Transaction<'_, Postgres>,
    ) -> Result<i32, sqlx::error::Error> {
        let row = sqlx::query_file!(
            "sql/insert_update/tarif.sql",
            self.msp_id,
            self.relationship_id,
            self.slug_name,
            self.monthly_fee
        )
        .fetch_one(&mut *transaction)
        .await?;

        Ok(row.id)
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
