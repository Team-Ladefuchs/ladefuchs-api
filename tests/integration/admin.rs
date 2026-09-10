use axum::http::{StatusCode, header::SET_COOKIE};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::helpers::TestClient;

// Successful login requires JWT_KEY to be set in the test environment.
#[sqlx::test]
async fn login_password_regression(pool: PgPool) {
    let hash = bcrypt::hash("correct-password", 4).unwrap();
    sqlx::query("INSERT INTO admin_user (username, password_hash) VALUES ($1, $2), ($3, $4)")
        .bind("login-regression")
        .bind(hash)
        .bind("malformed-hash")
        .bind("not-a-bcrypt-hash")
        .execute(&pool)
        .await
        .unwrap();

    let client = TestClient::new(pool).await;
    for (username, password) in [
        ("login-regression", "wrong-password"),
        ("unknown-user", "correct-password"),
        ("malformed-hash", "correct-password"),
    ] {
        let response = client
            .post(
                "/admin/login",
                json!({ "username": username, "password": password }),
            )
            .await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{username}");
        assert!(!response.headers().contains_key(SET_COOKIE), "{username}");
        assert_eq!(
            response.json::<Value>().await,
            json!({ "statusCode": 401, "reason": "wrong username or password" })
        );
    }

    let response = client
        .post(
            "/admin/login",
            json!({ "username": "login-regression", "password": "correct-password" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response
        .headers()
        .get(SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    let token = cookie
        .split(';')
        .next()
        .unwrap()
        .strip_prefix("auth_token=")
        .unwrap();
    assert!(!token.is_empty());
    let body = response.json::<Value>().await;
    assert_eq!(body["username"], "login-regression");
    assert!(body["exp"].as_i64().unwrap() > chrono::Utc::now().timestamp());
}
