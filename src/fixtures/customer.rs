use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct Customer {
    pub id: i32,
    pub pub_id: uuid::Uuid,
    pub name: String,
    pub total_impressions: i32,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Clone, Debug, Default)]
pub struct CustomerBuilder {
    pub_id: Option<uuid::Uuid>,
    name: Option<String>,
    total_impressions: i32,
}

impl CustomerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pub_id(mut self, pub_id: uuid::Uuid) -> Self {
        self.pub_id = Some(pub_id);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn total_impressions(mut self, total_impressions: i32) -> Self {
        self.total_impressions = total_impressions;
        self
    }

    pub async fn create(self, pool: &sqlx::PgPool) -> Customer {
        static NAME_SEQUENCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            pub_id: uuid::Uuid,
            name: String,
            total_impressions: i32,
            created: DateTime<Utc>,
            updated: DateTime<Utc>,
        }

        let name = self.name.unwrap_or_else(|| {
            format!(
                "Test Customer {}",
                NAME_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            )
        });

        let row: Row = sqlx::query_as(
            r#"
            INSERT INTO customer
                (pub_id, name, total_impressions)
            VALUES
                (COALESCE($1, gen_random_uuid()), $2, $3)
            RETURNING id, pub_id, name, total_impressions, created, updated
            "#,
        )
        .bind(self.pub_id)
        .bind(&name)
        .bind(self.total_impressions)
        .fetch_one(pool)
        .await
        .unwrap();

        Customer {
            id: row.id,
            pub_id: row.pub_id,
            name: row.name,
            total_impressions: row.total_impressions,
            created: row.created,
            updated: row.updated,
        }
    }
}
