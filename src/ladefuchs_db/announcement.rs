use sqlx::PgConnection;

struct Announcement {
    value: serde_json::Value,
}

pub async fn get_first_announcement(connection: &mut PgConnection) -> Option<serde_json::Value> {
    sqlx::query_as!(
        Announcement,
        r#"
            select value
            from announcement
            where (now() between start_at and end_at)
               or start_at is null
            order by start_at
            limit 1
        "#
    )
        .fetch_optional(connection)
        .await
        .ok()
        .flatten()
        .map(|result| result.value)
}
