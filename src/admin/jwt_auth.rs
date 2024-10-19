use crate::api::error::ApiError;
use crate::state::State;

use axum::Extension;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts, Json};
use axum_extra::extract::cookie::Cookie;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use time::OffsetDateTime;
use tower_cookies::cookie::SameSite;
use tower_cookies::Cookies;

pub(crate) const ADMIN_COOKIE_NAME: &'static str = "auth_token";

pub struct AdminAuthToken {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl AdminAuthToken {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }

    pub fn encode(user: &AdminUser) -> Result<std::string::String, jsonwebtoken::errors::Error> {
        encode(&Header::default(), user, &JWT_KEYS.encoding)
    }

    pub fn decode(token: &str) -> Result<TokenData<AdminUser>, jsonwebtoken::errors::Error> {
        decode::<AdminUser>(token, &JWT_KEYS.decoding, &Validation::default())
    }

    pub fn is_valid(token: &str) -> bool {
        decode::<AdminUser>(token, &JWT_KEYS.decoding, &Validation::default())
            .ok()
            .is_some()
    }
}

pub static JWT_KEYS: Lazy<AdminAuthToken> = Lazy::new(|| {
    // Fetch the secret from an environment variable (e.g., SECRET_KEY)
    let secret = env::var("JWT_KEY").expect("JWT_KEY must be set");
    AdminAuthToken::new(secret.as_bytes())
});

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUser {
    pub username: String,
    exp: i64,
}

impl AdminUser {
    fn new(username: String, expire: OffsetDateTime) -> Self {
        Self {
            username,
            exp: expire.unix_timestamp(),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookie = Cookies::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::LoginTimeOut)?;

        let token = cookie
            .get(ADMIN_COOKIE_NAME)
            .ok_or(ApiError::LoginTimeOut)?
            .value()
            .to_owned();

        let auth_token = AdminAuthToken::decode(&token).map_err(|_| ApiError::MissingToken)?;

        Ok(auth_token.claims)
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct User {
    id: i32,
    pub username: String,
    password_hash: String,
}

#[derive(Clone, serde::Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

pub async fn login(
    cookies: Cookies,
    Extension(state): Extension<State>,
    Json(credentials): Json<Credentials>,
) -> Result<Json<AdminUser>, ApiError> {
    let mut connection = state.database_pool.acquire().await?;

    let user: Option<User> = sqlx::query_as!(
        User,
        "SELECT id, username, password_hash FROM admin_user WHERE username = $1",
        credentials.username
    )
    .fetch_optional(&mut *connection) // Use a mutable reference here
    .await?;

    match user {
        Some(user)
            if bcrypt::verify(credentials.password, &user.password_hash)
                .ok()
                .is_some() =>
        {
            let expire = time::OffsetDateTime::now_utc() + time::Duration::weeks(3);

            let admin_user = AdminUser::new(user.username, expire);
            // Create the authorization token
            let token = AdminAuthToken::encode(&admin_user).map_err(|_| ApiError::Login)?;

            let cookie = Cookie::build((ADMIN_COOKIE_NAME, token))
                .domain(
                    state
                        .config
                        .admin_domain
                        .host()
                        .map(|h| h.to_string())
                        .unwrap_or_default(),
                )
                .path("/")
                .same_site(SameSite::Lax)
                .secure(cookie_secure())
                .expires(expire)
                .http_only(false)
                .build();

            cookies.add(cookie);

            Ok(Json(admin_user))
        }
        _ => Err(ApiError::Login),
    }
}

pub async fn confirm_login(admin_user: AdminUser) -> axum::Json<AdminUser> {
    Json(admin_user)
}

pub async fn logout(cookies: Cookies) -> Result<(), ApiError> {
    if let Some(mut cookie) = cookies.get(ADMIN_COOKIE_NAME) {
        cookie.make_removal();
    }

    Ok(())
}

#[cfg(not(debug_assertions))]
const fn cookie_secure() -> bool {
    true
}

#[cfg(debug_assertions)]
const fn cookie_secure() -> bool {
    true
}
