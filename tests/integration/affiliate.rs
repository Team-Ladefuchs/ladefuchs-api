use axum::http::HeaderValue;
use ladefuchs_api::fixtures::link::LinkBuilder;
use pretty_assertions::assert_eq;
use sqlx::PgPool;

use crate::helpers::TestClient;

#[sqlx::test]
async fn test_redirect_affiliate_redirects(pool: PgPool) {
    let link = LinkBuilder::new().create(&pool).await;
    let client = TestClient::new(pool).await;

    let result = client.get(format!("/affiliate?url={}", link.source)).await;

    assert_eq!(308, result.status());
    assert_eq!(
        Some(&HeaderValue::from_static("https://example.com/")),
        result.headers().get("location")
    );
}
