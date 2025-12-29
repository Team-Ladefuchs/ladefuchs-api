use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Operator {
    pub id: i32,
    pub network: uuid::Uuid,
    pub pub_network: uuid::Uuid,
    pub name: String,
    pub slug_name: String,
    pub standard: bool,
    pub updated: DateTime<Utc>,
}

pub struct OperatorBuilder {
    network: Option<uuid::Uuid>,
    pub_network: Option<uuid::Uuid>,
    name: Option<String>,
    slug_name: Option<String>,
    standard: bool,
    url: Option<String>,
}

impl Default for OperatorBuilder {
    fn default() -> Self {
        Self {
            network: None,
            pub_network: None,
            name: None,
            slug_name: None,
            standard: true,
            url: None,
        }
    }
}

impl OperatorBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn network(mut self, network: uuid::Uuid) -> Self {
        self.network = Some(network);
        self
    }

    pub fn pub_network(mut self, pub_network: uuid::Uuid) -> Self {
        self.pub_network = Some(pub_network);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn slug_name(mut self, slug_name: impl Into<String>) -> Self {
        self.slug_name = Some(slug_name.into());
        self
    }

    pub fn standard(mut self, standard: bool) -> Self {
        self.standard = standard;
        self
    }

    pub fn url(mut self, url: impl Into<Option<String>>) -> Self {
        self.url = url.into();
        self
    }

    pub async fn create(self, pool: &PgPool) -> Operator {
        static NAME_SEQUENCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        static SLUG_SEQUENCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

        let network = self.network.unwrap_or_else(uuid::Uuid::new_v4);
        let pub_network = self.pub_network.unwrap_or_else(uuid::Uuid::new_v4);

        let name = self.name.unwrap_or_else(|| {
            let seq = NAME_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("test-operator-{}", seq)
        });

        let slug_name = self.slug_name.unwrap_or_else(|| {
            let seq = SLUG_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("Test Operator {}", seq)
        });

        sqlx::query_as(
            r#"
            INSERT INTO operator
                (network, pub_network, name, slug_name, standard, url)
            VALUES
                ($1, $2, $3, $4, $5, $6)
            RETURNING
                id, network, pub_network, name, slug_name, standard, updated
            "#,
        )
        .bind(network)
        .bind(pub_network)
        .bind(&name)
        .bind(&slug_name)
        .bind(self.standard)
        .bind(self.url)
        .fetch_one(pool)
        .await
        .expect("could not insert operator fixture")
    }
}
