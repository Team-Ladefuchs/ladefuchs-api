pub mod v3 {
    use sqlx::{Connection, PgConnection};

    use crate::db::banner::PlatformType;

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
        Ok(())
    }
}
