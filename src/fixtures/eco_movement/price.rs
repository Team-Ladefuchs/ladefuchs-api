use serde_json::json;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct EcoPriceStaging {
    pub id: String,
    pub tariff_id: uuid::Uuid,
    pub provider_name: String,
}

pub struct EcoPriceStagingBuilder {
    id: Option<String>,
    provider_name: Option<String>,
    tariff_id: uuid::Uuid,
    elements: Option<serde_json::Value>,
}

impl EcoPriceStagingBuilder {
    pub fn new(tariff_id: uuid::Uuid) -> Self {
        Self {
            id: None,
            provider_name: None,
            tariff_id,
            elements: None,
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn provider_name(mut self, provider_name: impl Into<String>) -> Self {
        self.provider_name = Some(provider_name.into());
        self
    }

    pub fn elements(mut self, elements: serde_json::Value) -> Self {
        self.elements = Some(elements);
        self
    }

    pub fn energy_only(mut self, price_excl_vat: f64) -> Self {
        self.elements = Some(json!([
            {
                "price_components": [
                    {
                        "price_excl_vat": price_excl_vat,
                        "vat": 19,
                        "step_size": 1,
                        "price_type": "ENERGY",
                    }
                ],
                "restrictions": null,
            }
        ]));

        self
    }

    pub fn energy_with_parking(
        mut self,
        price_excl_vat: f64,
        min_duration: i32,
        parking_excl_vat: f64,
    ) -> Self {
        self.elements = Some(json!([
            {
                "price_components": [
                    {
                        "price_excl_vat": price_excl_vat,
                        "vat": 19,
                        "step_size": 1,
                        "price_type": "ENERGY",
                    },
                    {
                        "price_excl_vat": parking_excl_vat,
                        "vat": 19,
                        "step_size": 1,
                        "price_type": "PARKING_TIME",
                    }
                ],
                "restrictions": { "min_duration": min_duration },
            }
        ]));

        self
    }

    pub async fn create(self, pool: &PgPool) -> EcoPriceStaging {
        static ID_SEQUENCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        static PROVIDER_SEQUENCE: std::sync::atomic::AtomicU32 =
            std::sync::atomic::AtomicU32::new(0);

        let id = self.id.unwrap_or_else(|| {
            let seq = ID_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("price-{}", seq)
        });

        let provider_name = self.provider_name.unwrap_or_else(|| {
            let seq = PROVIDER_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("Eco Provider {}", seq)
        });

        let elements = self.elements.unwrap_or_else(|| json!([]));

        sqlx::query(
            "INSERT INTO eco_movement.price (id, provider_name, tariff_id, elements) VALUES ($1, $2, $3, $4)",
        )
        .bind(&id)
        .bind(&provider_name)
        .bind(self.tariff_id)
        .bind(elements)
        .execute(pool)
        .await
        .expect("could not insert eco_movement.price fixture");

        EcoPriceStaging {
            id,
            tariff_id: self.tariff_id,
            provider_name,
        }
    }
}
