pub mod v3 {
    use crate::{
        api::{json, ApiJson},
        db::{self, banner::PlatformType},
        state::State,
    };

    use axum::{Extension, Json};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AppMetricRequest {
        pub device_id: Option<uuid::Uuid>,
        pub plattform: PlatformType,
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

        db::app_metrics::v3::insert(
            &mut connection,
            &app_id,
            &request.plattform,
            &i32::from(request.version),
        )
        .await?;

        json(AppMetricResponse { device_id: app_id })
    }
}
