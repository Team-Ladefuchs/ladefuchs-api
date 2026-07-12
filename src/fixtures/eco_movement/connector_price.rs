use sqlx::PgPool;

pub struct EcoConnectorPriceBuilder {
    location_id: uuid::Uuid,
    pricing_id: String,
    evse_uid: String,
    connector_id: String,
}

impl EcoConnectorPriceBuilder {
    pub fn new(
        location_id: uuid::Uuid,
        pricing_id: impl Into<String>,
        evse_uid: impl Into<String>,
        connector_id: impl Into<String>,
    ) -> Self {
        Self {
            location_id,
            pricing_id: pricing_id.into(),
            evse_uid: evse_uid.into(),
            connector_id: connector_id.into(),
        }
    }

    pub async fn create(self, pool: &PgPool) {
        sqlx::query(
            "INSERT INTO eco_movement.connector_price (location_id, pricing_id, evse_uid, connector_id) VALUES ($1, $2, $3, $4)",
        )
        .bind(self.location_id)
        .bind(&self.pricing_id)
        .bind(&self.evse_uid)
        .bind(&self.connector_id)
        .execute(pool)
        .await
        .expect("could not insert eco_movement.connector_price fixture");
    }
}
