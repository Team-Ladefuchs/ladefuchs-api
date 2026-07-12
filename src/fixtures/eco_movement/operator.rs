use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct EcoOperator {
    pub id: uuid::Uuid,
    pub name: String,
}

#[derive(Default)]
pub struct EcoOperatorBuilder {
    id: Option<uuid::Uuid>,
    name: Option<String>,
    website: Option<String>,
    ema_id: Vec<String>,
}

impl EcoOperatorBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: uuid::Uuid) -> Self {
        self.id = Some(id);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn website(mut self, website: impl Into<String>) -> Self {
        self.website = Some(website.into());
        self
    }

    pub fn ema_id(mut self, ema_id: Vec<String>) -> Self {
        self.ema_id = ema_id;
        self
    }

    pub async fn create(self, pool: &PgPool) -> EcoOperator {
        static NAME_SEQUENCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

        let id = self.id.unwrap_or_else(uuid::Uuid::new_v4);
        let name = self.name.unwrap_or_else(|| {
            let seq = NAME_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("Eco Operator {}", seq)
        });

        sqlx::query(
            "INSERT INTO eco_movement.operator (id, name, website, ema_id) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(&name)
        .bind(&self.website)
        .bind(&self.ema_id)
        .execute(pool)
        .await
        .expect("could not insert eco_movement.operator fixture");

        EcoOperator { id, name }
    }
}
