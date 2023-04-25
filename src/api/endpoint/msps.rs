use axum::Extension;

use crate::{
    api::{json_list, ApiJsonList},
    db::{self, msp},
    state::State,
};

pub async fn get_all(Extension(state): Extension<State>) -> ApiJsonList<msp::Msp> {
    let mut connection = state.database_pool.acquire().await?;
    let msp_list = db::msp::get_all(&mut connection).await?;

    json_list(msp_list)
}
