use sqlx::PgPool;

use crate::eco_movement::api::response::location::{ConnectorType, PowerType};

#[derive(Debug, Clone)]
pub struct EcoConnectorStaging {
    pub id: String,
    pub evse_uid: String,
}

pub struct EcoConnectorBuilder {
    id: Option<String>,
    evse_uid: Option<String>,
    power_type: PowerType,
    max_power: i32,
    connector_type: ConnectorType,
}

impl Default for EcoConnectorBuilder {
    fn default() -> Self {
        Self {
            id: None,
            evse_uid: None,
            power_type: PowerType::Dc,
            max_power: 150,
            connector_type: ConnectorType::CCS,
        }
    }
}

impl EcoConnectorBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn evse_uid(mut self, evse_uid: impl Into<String>) -> Self {
        self.evse_uid = Some(evse_uid.into());
        self
    }

    pub fn power_type(mut self, power_type: PowerType) -> Self {
        self.power_type = power_type;
        self
    }

    pub fn max_power(mut self, max_power: i32) -> Self {
        self.max_power = max_power;
        self
    }

    pub fn connector_type(mut self, connector_type: ConnectorType) -> Self {
        self.connector_type = connector_type;
        self
    }

    pub async fn create(self, pool: &PgPool) -> EcoConnectorStaging {
        static ID_SEQUENCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        static EVSE_SEQUENCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

        let id = self.id.unwrap_or_else(|| {
            let seq = ID_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("connector-{}", seq)
        });
        let evse_uid = self.evse_uid.unwrap_or_else(|| {
            let seq = EVSE_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("evse-{}", seq)
        });

        sqlx::query(
            "INSERT INTO eco_movement.connector (id, evse_uid, power_type, max_power, connector_type) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&id)
        .bind(&evse_uid)
        .bind(self.power_type)
        .bind(self.max_power)
        .bind(self.connector_type)
        .execute(pool)
        .await
        .expect("could not insert eco_movement.connector fixture");

        EcoConnectorStaging { id, evse_uid }
    }
}
