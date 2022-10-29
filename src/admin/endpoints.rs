use axum::{
    extract::{Json, Path},
    Extension,
};

use cookie::{time::Duration, SameSite};
use once_cell::sync::Lazy;
use rand::RngCore;
use tower_cookies::{Cookie, Cookies, Key};

use crate::{
    api::{
        error::{self, ApiError},
        util::{json, json_list},
        ApiJson, ApiJsonList,
    },
    db::{
        self,
        banner::{banner_click_statistics, banner_click_summary, ClicksPerDay, ThgClickSummery},
        charge_price::ImportResult,
        cpo::CPO,
        tariff::TariffIntern,
    },
    importer::{self, import},
    state::State,
};

pub const COOKIE_NAME: &str = "auth";

pub static COOKIE_KEY: Lazy<Key> = Lazy::new(|| {
    let mut buf = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut buf);
    Key::from(&buf)
});

#[derive(Clone, serde::Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Clone, serde::Serialize)]
pub struct AdminUser {
    username: String,
}

pub async fn login(
    Json(credentials): Json<Credentials>,
    Extension(state): Extension<State>,
    cookies: Cookies,
) -> Result<axum::Json<AdminUser>, error::ApiError> {
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

pub async fn verify_login(cookies: Cookies) -> Result<axum::Json<AdminUser>, error::ApiError> {
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
    Path(days): Path<i32>,
) -> Result<ApiJsonList<ClicksPerDay>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let clicks = banner_click_statistics(&mut connection, days, 3).await?;
    Ok(json_list(clicks))
}

pub async fn get_banner_statistics(
    Extension(state): Extension<State>,
) -> Result<ApiJson<ThgClickSummery>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let summary = banner_click_summary(&mut connection, 3).await?;
    Ok(json(summary))
}

pub async fn get_all_cpos(
    Extension(state): Extension<State>,
) -> Result<ApiJsonList<CPO>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let cpos = db::cpo::get_all(&mut connection).await?;

    Ok(json_list(cpos))
}

pub async fn last_import(
    Extension(state): Extension<State>,
) -> Result<ApiJson<ImportResult>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let import_result =
        db::charge_price::import_meta(&mut connection, state.config.interval).await?;
    Ok(json(import_result))
}

pub async fn trigger_import(
    Extension(state): Extension<State>,
) -> Result<ApiJson<ImportResult>, error::ApiError> {
    import(&state, importer::Mode::Manual).await?;
    let mut connection = state.database_pool.acquire().await?;
    let import_result =
        db::charge_price::import_meta(&mut connection, state.config.interval).await?;
    tracing::info!(status = "manual import finished!", prices=import_result.prices, last_updated= ?import_result.last_import);
    Ok(json(import_result))
}
