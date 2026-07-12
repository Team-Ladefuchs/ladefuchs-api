use sqlx::PgPool;

use crate::eco_movement::api::response::tariff::TariffType;

#[derive(Debug, Clone)]
pub struct EcoTariffStaging {
    pub id: uuid::Uuid,
    pub product_id: Option<uuid::Uuid>,
    pub name: String,
    pub provider_name: String,
}

pub struct EcoTariffStagingBuilder {
    id: Option<uuid::Uuid>,
    product_id: Option<uuid::Uuid>,
    name: Option<String>,
    description: Option<String>,
    subscription_type: Option<String>,
    tariff_type: TariffType,
    subscription_fee_excl_vat: String,
    currency: String,
    provider_name: Option<String>,
}

impl Default for EcoTariffStagingBuilder {
    fn default() -> Self {
        Self {
            id: None,
            product_id: Some(uuid::Uuid::new_v4()),
            name: None,
            description: None,
            subscription_type: None,
            tariff_type: TariffType::Msp,
            subscription_fee_excl_vat: "0".to_string(),
            currency: "EUR".to_string(),
            provider_name: None,
        }
    }
}

impl EcoTariffStagingBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: uuid::Uuid) -> Self {
        self.id = Some(id);
        self
    }

    pub fn product_id(mut self, product_id: Option<uuid::Uuid>) -> Self {
        self.product_id = product_id;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn subscription_type(mut self, subscription_type: impl Into<String>) -> Self {
        self.subscription_type = Some(subscription_type.into());
        self
    }

    pub fn tariff_type(mut self, tariff_type: TariffType) -> Self {
        self.tariff_type = tariff_type;
        self
    }

    pub fn subscription_fee_excl_vat(mut self, fee: impl Into<String>) -> Self {
        self.subscription_fee_excl_vat = fee.into();
        self
    }

    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = currency.into();
        self
    }

    pub fn provider_name(mut self, provider_name: impl Into<String>) -> Self {
        self.provider_name = Some(provider_name.into());
        self
    }

    pub async fn create(self, pool: &PgPool) -> EcoTariffStaging {
        static NAME_SEQUENCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        static PROVIDER_SEQUENCE: std::sync::atomic::AtomicU32 =
            std::sync::atomic::AtomicU32::new(0);

        let id = self.id.unwrap_or_else(uuid::Uuid::new_v4);

        let name = self.name.unwrap_or_else(|| {
            let seq = NAME_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("Eco Tariff {}", seq)
        });

        let provider_name = self.provider_name.unwrap_or_else(|| {
            let seq = PROVIDER_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("Eco Provider {}", seq)
        });

        sqlx::query(
            "INSERT INTO eco_movement.tariff (id, name, description, subscription_type, type, subscription_fee_excl_vat, currency, provider_name, product_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(id)
        .bind(&name)
        .bind(&self.description)
        .bind(&self.subscription_type)
        .bind(self.tariff_type)
        .bind(&self.subscription_fee_excl_vat)
        .bind(&self.currency)
        .bind(&provider_name)
        .bind(self.product_id)
        .execute(pool)
        .await
        .expect("could not insert eco_movement.tariff fixture");

        EcoTariffStaging {
            id,
            product_id: self.product_id,
            name,
            provider_name,
        }
    }
}
