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

        db::app_metrics::v3::insert_or_update(&mut connection, &app_id, &request.plattform).await?;

        json(AppMetricResponse { device_id: app_id })
    }
}
