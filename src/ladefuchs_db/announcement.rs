use sqlx::PgConnection;

struct Announcement {
    value: serde_json::Value,
}

pub async fn get_first_announcement(connection: &mut PgConnection) -> Option<serde_json::Value> {
    let ret = sqlx::query_file_as!(Announcement, "sql/get/first_anouncemnt.sql",)
        .fetch_optional(connection)
        .await
        .ok()
        .flatten()
        .map(|result| result.value);
    return ret;
}
