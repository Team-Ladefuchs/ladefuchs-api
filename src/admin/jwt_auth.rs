use jsonwebtoken::{DecodingKey, Validation, decode};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env;

pub(crate) const ADMIN_COOKIE_NAME: &str = "auth_token";

pub struct AdminAuthToken {
    decoding: DecodingKey,
}

#[derive(Deserialize)]
struct AdminClaims {
    username: String,
    exp: i64,
}

impl AdminAuthToken {
    fn new(secret: &[u8]) -> Self {
        Self {
            decoding: DecodingKey::from_secret(secret),
        }
    }

    pub fn is_valid(token: &str) -> bool {
        decode::<AdminClaims>(token, &JWT_KEYS.decoding, &Validation::default())
            .map(|token| !token.claims.username.is_empty() && token.claims.exp > 0)
            .unwrap_or(false)
    }
}

pub static JWT_KEYS: Lazy<AdminAuthToken> = Lazy::new(|| {
    let secret = env::var("JWT_KEY").expect("JWT_KEY must be set");
    AdminAuthToken::new(secret.as_bytes())
});
