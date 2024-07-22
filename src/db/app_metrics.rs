pub mod v3 {
    use sqlx::{Connection, PgConnection};

    use crate::db::banner::PlatformType;

    pub async fn insert_or_update(
        connection: &mut PgConnection,
        app_id: &uuid::Uuid,
        platform: &PlatformType,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = connection.begin().await?;
        sqlx::query_file!("sql/insert/app_metrics.sql", app_id, platform as _)
            .execute(&mut *transaction)
            .await?;
        Ok(())
    }
}
