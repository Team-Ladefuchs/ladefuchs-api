use axum::{extract::Json, Extension};
use cookie::{time::Duration, SameSite};
use once_cell::sync::Lazy;
use rand::RngCore;
use tower_cookies::{Cookie, Cookies, Key};

use crate::{
    api::{
        error::{self, ApiError},
        util::{json, json_list},
        ApiJsonList,
    },
    db::{self, cpo::CPO, tariff::TariffIntern},
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

pub async fn get_all_cpos(
    Extension(state): Extension<State>,
) -> Result<ApiJsonList<CPO>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let cpos = db::cpo::get_all(&mut connection).await?;

    Ok(json_list(cpos))
}
