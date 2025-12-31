use axum::http::HeaderValue;
use ladefuchs_api::fixtures::{banner::BannerBuilder, link::LinkBuilder};
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

#[sqlx::test]
async fn test_redirect_affiliate_redirects_with_banner(pool: PgPool) {
    let link = LinkBuilder::new().create(&pool).await;
    let banner = BannerBuilder::new().link_id(link.id).create(&pool).await;
    let client = TestClient::new(pool).await;

    let result = client
        .get(format!(
            "/affiliate?url={}&banner={}",
            link.source, banner.identifier
        ))
        .await;

    assert_eq!(308, result.status());
    assert_eq!(
        Some(&HeaderValue::from_static("https://example.com/")),
        result.headers().get("location")
    );
}

#[sqlx::test]
async fn test_redirect_affiliate_not_found(pool: PgPool) {
    let client = TestClient::new(pool).await;

    let result = client
        .get("/affiliate?url=https%3A%2F%2Fnonexistent.example.com%2F")
        .await;

    assert_eq!(404, result.status());
}

#[sqlx::test]
async fn test_redirect_affiliate_missing_url_param(pool: PgPool) {
    let client = TestClient::new(pool).await;

    let result = client.get("/affiliate").await;

    assert_eq!(400, result.status());
}

#[sqlx::test]
async fn test_redirect_affiliate_empty_url_param(pool: PgPool) {
    let client = TestClient::new(pool).await;

    let result = client.get("/affiliate?url=").await;

    assert_eq!(400, result.status());
}

#[sqlx::test]
async fn test_redirect_affiliate_empty_banner_param(pool: PgPool) {
    let link = LinkBuilder::new().create(&pool).await;
    let client = TestClient::new(pool).await;

    let result = client
        .get(format!("/affiliate?url={}&banner=", link.source))
        .await;

    assert_eq!(400, result.status());
}

#[sqlx::test]
async fn test_redirect_affiliate_invalid_url(pool: PgPool) {
    let client = TestClient::new(pool).await;

    let result = client.get("/affiliate?url=invalid-url").await;

    assert_eq!(400, result.status());
}

#[sqlx::test]
async fn test_redirect_affiliate_persists_tracking_row(pool: PgPool) {
    let link = LinkBuilder::new().create(&pool).await;
    let client = TestClient::new(pool.clone()).await;

    let result = client
        .get_with_user_agent(
            format!("/affiliate?url={}", link.source),
            "Mozilla/5.0 (iPhone)",
        )
        .await;

    assert_eq!(308, result.status());

    let count: i64 =
        sqlx::query_scalar("select count(1) from affiliate_statistic where link_id = $1")
            .bind(link.id)
            .fetch_one(&pool)
            .await
            .expect("could not query affiliate_statistic");

    assert_eq!(1, count);
}

#[sqlx::test]
async fn test_redirect_affiliate_persists_tracking_row_with_banner(pool: PgPool) {
    let link = LinkBuilder::new().create(&pool).await;
    let banner = BannerBuilder::new().link_id(link.id).create(&pool).await;
    let client = TestClient::new(pool.clone()).await;

    let result = client
        .get_with_user_agent(
            format!(
                "/affiliate?url={}&banner={}",
                link.source, banner.identifier
            ),
            "Mozilla/5.0 (Linux; Android 14)",
        )
        .await;

    assert_eq!(308, result.status());

    let rows = sqlx::query("SELECT * FROM affiliate_statistic WHERE link_id = $1")
        .bind(link.id)
        .fetch_all(&pool)
        .await
        .expect("could not query affiliate_statistic");
    dbg!(&rows);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM affiliate_statistic WHERE link_id = $1 AND link_banner_id IS NOT NULL")
    .bind(link.id)
    .fetch_one(&pool)
    .await
    .expect("could not query affiliate_statistic");

    assert_eq!(1, count);
}
