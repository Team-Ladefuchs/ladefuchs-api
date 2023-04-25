use axum::{
    extract::{Json, Path},
    Extension,
};
use once_cell::sync::Lazy;
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies, Key,
};

use crate::{
    api::{
        error::{self, ApiError},
        json, json_list, ApiJson, ApiJsonList,
    },
    db::{
        self,
        banner::{banner_click_statistics, banner_click_summary, ClicksPerDay, ThgClickSummery},
        charge_price::{AdminImportResult, ImportStatus},
        cpo::{self, get_by_internal_id, has_no_prices, CPO},
        cpo_cache::{self, CPOCache},
        tariff::TariffIntern,
    },
    importer,
    slack::SlackClient,
    state::State,
};

pub const COOKIE_NAME: &str = "auth";

pub static COOKIE_KEY: Lazy<Key> = Lazy::new(|| Key::generate());

#[derive(Clone, serde::Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Clone, serde::Serialize)]
pub struct AdminUser {
    username: String,
}
// #[axum::debug_handler]
pub async fn login(
    Extension(state): Extension<State>,
    cookies: Cookies,
    Json(credentials): Json<Credentials>,
) -> Result<Json<AdminUser>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let db_credentials = db::user::get_admin(&mut connection, &credentials.username).await?;

    match db_credentials {
        Some(user) if bcrypt::verify(credentials.password, &user.password).unwrap() => {
            let private_cookies = cookies.private(&COOKIE_KEY);
            let username = user.username;
            let mut cookie_builder =
                Cookie::build(COOKIE_NAME, username.clone()).same_site(SameSite::Lax);
            cookie_builder = cookie_builder.domain(
                state
                    .as_ref()
                    .config
                    .admin_domain
                    .host_str()
                    .map(|host| host.replace("admin.", ""))
                    .unwrap_or_default(),
            );

            private_cookies.add(
                cookie_builder
                    .max_age(Duration::days(10))
                    .path("/")
                    .secure(true)
                    .finish(),
            );
            json(AdminUser { username })
        }
        _ => Err(ApiError::Login),
    }
}

pub async fn confirm_login(cookies: Cookies) -> Result<axum::Json<AdminUser>, error::ApiError> {
    let cookie = cookies
        .private(&COOKIE_KEY)
        .get(COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());
    match cookie {
        Some(username) => json(AdminUser { username }),
        None => Err(ApiError::LoginTimeOut),
    }
}

pub async fn logout(cookies: Cookies) -> Result<(), error::ApiError> {
    let private_cookies = cookies.private(&COOKIE_KEY);
    private_cookies.remove(Cookie::new("auth", ""));

    Ok(())
}

pub async fn get_all_tariffs(
    Extension(state): Extension<State>,
) -> Result<ApiJsonList<TariffIntern>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let tariffs = db::tariff::get_all_intern(&mut connection).await?;

    Ok(json_list(tariffs))
}

pub async fn get_banner_chart_data(
    Extension(state): Extension<State>,
    Path((days, link_id)): Path<(i32, i32)>,
) -> Result<ApiJsonList<ClicksPerDay>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let clicks = banner_click_statistics(&mut connection, days, link_id).await?;
    Ok(json_list(clicks))
}

pub async fn get_banner_statistics(
    Extension(state): Extension<State>,
    Path(link_id): Path<i32>,
) -> Result<ApiJson<ThgClickSummery>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let summary = banner_click_summary(&mut connection, link_id).await?;
    Ok(json(summary))
}

pub async fn get_all_cpos(
    Extension(state): Extension<State>,
) -> Result<ApiJsonList<CPO>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let cpos = db::cpo::get_all(&mut connection).await?;

    Ok(json_list(cpos))
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CpoSearchRequest {
    query: String,
}

pub async fn cpo_search(
    Extension(state): Extension<State>,
    Json(request): Json<CpoSearchRequest>,
) -> Result<ApiJsonList<CPOCache>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let result = cpo_cache::search(&mut connection, &request.query).await?;
    Ok(json(result))
}

pub async fn delete_cpo(
    Extension(state): Extension<State>,
    Path(cpo_id): Path<i32>,
) -> Result<(), error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    cpo::delete_by_id(&mut connection, cpo_id).await?;
    Ok(())
}

pub async fn insert_update_cpo(
    Extension(state): Extension<State>,
    Json(new_cpo): Json<CPO>,
) -> Result<(), error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    db::cpo_cache::get_by_network(&mut connection, &new_cpo.network)
        .await
        .map_err(|e| {
            tracing::debug!("insert_update_cpo: {:?}", e);
            error::ApiError::CpoNotFound(format!(
                "uuid: {}, name: {}",
                new_cpo.network, new_cpo.slug_name
            ))
        })?;

    let cpo_id = new_cpo.insert_or_update(&mut connection).await?;

    if new_cpo.is_enabled && has_no_prices(&mut connection, cpo_id).await? {
        let cpo = get_by_internal_id(&mut connection, cpo_id).await?;
        state
            .import_prices(&mut connection, importer::Mode::Manual, &[cpo])
            .await?;
    }

    Ok(())
}

pub async fn last_import(
    Extension(state): Extension<State>,
) -> Result<ApiJson<AdminImportResult>, error::ApiError> {
    let status = ImportStatus::from(state.is_import_locked());
    let import_result = match status {
        ImportStatus::Waiting => {
            let mut connection = state.database_pool.acquire().await?;
            let interval_time = state.timer.next().await?;
            let import_result =
                db::charge_price::import_metadata(&mut connection, Some(interval_time)).await?;
            Some(import_result)
        }
        ImportStatus::InProgress => None,
    };

    Ok(json(AdminImportResult {
        status,
        import_result,
    }))
}

pub async fn trigger_manual_import(
    Extension(state): Extension<State>,
) -> Result<(), error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let cpo_list = cpo::get_with(&mut connection, cpo::Filter::Enabled).await?;

    if state.is_import_locked() {
        return Err(ApiError::ImportInProgress);
    }

    tokio::task::spawn(async move {
        let slack = &state.slack;

        match state
            .import_prices(&mut connection, importer::Mode::Manual, &cpo_list)
            .await
        {
            Ok(prices_count) => {
                slack
                    .send(
                        None,
                        &format!("Manual import was successful. Prices: {}", prices_count),
                    )
                    .await;
            }
            Err(err) => {
                slack
                    .send(
                        None,
                        &format!("Error occurred during manual import: {}", err),
                    )
                    .await;
            }
        };
    });

    Ok(())
}
