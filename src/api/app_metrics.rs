use serde::{Deserialize, Serialize};

pub mod v3 {
    use super::*;
    use crate::{
        api::{ApiJson, json},
        ladefuchs_db::{self, banner::PlatformType},
        state::State,
    };

    use axum::{Extension, Json};

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AppMetricRequest {
        pub device_id: Option<uuid::Uuid>,
        pub platform: PlatformType,
        pub version: u16,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AppMetricResponse {
        pub device_id: uuid::Uuid,
    }

    pub async fn post_handler(
        Extension(state): Extension<State>,
        Json(request): Json<AppMetricRequest>,
    ) -> ApiJson<AppMetricResponse> {
        let app_id = match request.device_id {
            Some(id) => id,
            None => uuid::Uuid::now_v7(),
        };
        let mut connection = state.database_pool.acquire().await?;

        ladefuchs_db::app_metrics::v3::insert(
            &mut connection,
            &app_id,
            &request.platform,
            &i32::from(request.version),
        )
        .await?;

        json(AppMetricResponse { device_id: app_id })
    }
}

pub mod admin {
    use crate::ladefuchs_db::app_metrics::admin::{AppUsageByPlatform, AppUsageGroupByDay};

    use super::*;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AppMetricsResponse {
        pub usage_by_platform: AppUsageByPlatform,
        pub usage_group_by_day: Vec<AppUsageGroupByDay>,
        pub total_banner_impression: i64,
    }
}
