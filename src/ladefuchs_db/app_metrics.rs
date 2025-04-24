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

pub mod admin {
    use serde::Serialize;
    use sqlx::postgres;

    use super::*;

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct AppUsageByPlatform {
        ios: i64,
        android: i64,
        total: i64,
    }

    pub async fn app_usage_number_by_platform(
        connection: &mut PgConnection,
        days: i32,
    ) -> Result<AppUsageByPlatform, sqlx::Error> {
        let interval = postgres::types::PgInterval {
            months: 0,
            days,
            microseconds: 0,
        };
        sqlx::query_file_as!(
            AppUsageByPlatform,
            "sql/get/app_metrics/usage_number_last_days.sql",
            interval
        )
        .fetch_one(connection)
        .await
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct AppUsageGroupByDay {
        ios: i64,
        android: i64,
        total: i64,
        visit_date: sqlx::types::chrono::NaiveDate,
    }

    pub async fn app_usage_group_by_day(
        connection: &mut PgConnection,
        days: i32,
    ) -> Result<Vec<AppUsageGroupByDay>, sqlx::Error> {
        let interval = postgres::types::PgInterval {
            months: 0,
            days,
            microseconds: 0,
        };
        sqlx::query_file_as!(
            AppUsageGroupByDay,
            "sql/get/app_metrics/usage_historic_group_by_day.sql",
            interval
        )
        .fetch_all(connection)
        .await
    }

    pub async fn banner_impression_last_days(
        connection: &mut PgConnection,
        days: i32,
    ) -> Result<i64, sqlx::Error> {
        let interval = postgres::types::PgInterval {
            months: 0,
            days,
            microseconds: 0,
        };
        sqlx::query_file_scalar!("sql/get/banner/banner_impression_last_days.sql", interval)
            .fetch_one(connection)
            .await
    }
}
