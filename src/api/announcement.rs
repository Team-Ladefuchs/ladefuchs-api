use axum::Extension;

pub mod v3 {

    use crate::{
        api::{ApiJson, json},
        ladefuchs_db::announcement,
        state::State,
    };

    use super::*;
    pub async fn get_handler(
        Extension(state): Extension<State>,
    ) -> ApiJson<Option<serde_json::Value>> {
        let mut connection = state.database_pool.acquire().await?;
        json(announcement::get_first_announcement(&mut connection).await)
    }
}
