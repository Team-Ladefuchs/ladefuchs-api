use axum::{async_trait, Json};
use axum_login::{AuthUser, AuthnBackend, UserId};
use sqlx::{FromRow, Pool, Postgres};

use crate::api::{
    error::{self, ApiError},
    json,
};

#[derive(Clone, serde::Serialize, serde::Deserialize, FromRow)]
pub struct User {
    id: i32,
    pub username: String,
    password_hash: String,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .finish()
    }
}

impl AuthUser for User {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: Pool<Postgres>,
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = sqlx::Error;

    async fn authenticate(
        &self,
        credentials: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<Self::User> = sqlx::query_as(
            "select id, username, password_hash from admin_user where username = $1",
        )
        .bind(credentials.username)
        .fetch_optional(&self.db)
        .await?;
        Ok(user.filter(|user| {
            bcrypt::verify(credentials.password, &user.password_hash)
                .ok()
                .is_some()
        }))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user =
            sqlx::query_as("select id, username, password_hash from admin_user where id = $1")
                .bind(user_id)
                .fetch_optional(&self.db)
                .await?;

        Ok(user)
    }
}

impl Backend {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }
}
pub type AuthSession = axum_login::AuthSession<Backend>;

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
    mut auth_session: AuthSession,
    Json(credentials): Json<Credentials>,
) -> Result<Json<AdminUser>, error::ApiError> {
    match auth_session.authenticate(credentials.clone()).await {
        Ok(Some(user)) => {
            if auth_session.login(&user).await.is_err() {
                return Err(ApiError::Login);
            }

            let username = user.username;

            json(AdminUser { username })
        }
        _ => Err(ApiError::Login),
    }
}

pub async fn confirm_login(
    auth_session: AuthSession,
) -> Result<axum::Json<AdminUser>, error::ApiError> {
    match auth_session.user {
        Some(user) => json(AdminUser {
            username: user.username,
        }),
        None => Err(ApiError::LoginTimeOut),
    }
}

pub async fn logout(mut auth_session: AuthSession) -> Result<(), error::ApiError> {
    if let Err(error) = auth_session.logout() {
        tracing::error!(%error, "Admin logout issue");
    }
    Ok(())
}
