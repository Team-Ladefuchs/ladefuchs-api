use crate::ladefuchs_db::banner::PlatformType;
use sqlx::{Connection, PgConnection};

pub mod v3 {
    use super::*;

    pub async fn insert(
        connection: &mut PgConnection,
        app_id: &uuid::Uuid,
        platform: &PlatformType,
        version: &i32,
    ) -> Result<(), sqlx::Error> {
        let mut transaction: sqlx::Transaction<sqlx::Postgres> = connection.begin().await?;
        sqlx::query_file!("sql/insert/app_metrics.sql", app_id, platform as _, version)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(())
    }
}
