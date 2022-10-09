use axum::{
    body::Body,
    extract::{rejection::QueryRejection, Query},
    http::Request,
    response::Redirect,
    Extension,
};
use serde::Deserialize;

use crate::{
    api::error::ApiError,
    db::banner::{self, PlatformType},
    state::State,
};

#[derive(Deserialize, Debug)]
pub struct AffilateParams {
    url: url::Url,
    banner: Option<uuid::Uuid>,
}

pub async fn redirect_affiliate(
    Extension(state): Extension<State>,
    params: Result<Query<AffilateParams>, QueryRejection>,
    req: Request<Body>,
) -> Result<Redirect, ApiError> {
    let parameter = params?;
    let url = &parameter.url;
    let mut connection = state.database_pool.acquire().await?;

    let banner_row = if let Some(banner) = &parameter.banner {
        banner::get_by_id(&mut connection, banner).await
    } else {
        None
    };

    match banner::link_id(&mut connection, url).await {
        Some(id) => {
            let user_agent = &req
                .headers()
                .get("user-agent")
                .map(|header| header.to_str().unwrap_or_default())
                .map(|agent| PlatformType::from(agent));
            if let Some(platform) = user_agent {
                let result = banner::update_link_states(
                    &mut connection,
                    id,
                    platform,
                    banner_row.map(|(id, _)| id),
                )
                .await;
                if let Err(error) = result {
                    tracing::warn!(msg = "Update affilate link statistics", %error);
                }
            }
        }
        None => return Err(ApiError::BadRequest),
    }

    Ok(Redirect::permanent(url.as_str()))
}
