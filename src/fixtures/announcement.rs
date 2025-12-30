use serde_json::Value;
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Announcement {
    pub id: uuid::Uuid,
    pub value: Value,
}

#[derive(Clone, Debug)]
pub struct AnnouncementBuilder {
    value: Option<Value>,
}

impl Default for AnnouncementBuilder {
    fn default() -> Self {
        Self {
            value: Some(serde_json::json!({
                "title": "Test Announcement",
                "message": "This is a test announcement"
            })),
        }
    }
}

impl AnnouncementBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn value(mut self, value: Value) -> Self {
        self.value = Some(value);
        self
    }

    pub async fn create(self, pool: &PgPool) -> Announcement {
        let value = self.value.expect("value must be set");

        sqlx::query_as("INSERT INTO announcement (value) VALUES ($1) RETURNING *")
            .bind(&value)
            .fetch_one(pool)
            .await
            .expect("could not insert announcement fixture")
    }
}
