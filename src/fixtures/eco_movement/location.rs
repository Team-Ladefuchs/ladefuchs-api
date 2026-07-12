use serde_json::json;
use sqlx::PgPool;

use crate::eco_movement::api::response::location::LocationType;

#[derive(Debug, Clone)]
pub struct EcoLocationStaging {
    pub id: uuid::Uuid,
    pub operator_id: uuid::Uuid,
}

pub struct EcoLocationBuilder {
    id: Option<uuid::Uuid>,
    operator_id: uuid::Uuid,
    location_type: LocationType,
    latitude: f64,
    longitude: f64,
    address: Option<String>,
    city: Option<String>,
    postal_code: Option<String>,
    extra_value: Option<serde_json::Value>,
}

impl EcoLocationBuilder {
    pub fn new(operator_id: uuid::Uuid) -> Self {
        Self {
            id: None,
            operator_id,
            location_type: LocationType::OnStreet,
            latitude: 52.5200,
            longitude: 13.4050,
            address: Some("Teststraße 1".to_string()),
            city: Some("Berlin".to_string()),
            postal_code: Some("10115".to_string()),
            extra_value: None,
        }
    }

    pub fn id(mut self, id: uuid::Uuid) -> Self {
        self.id = Some(id);
        self
    }

    pub fn location_type(mut self, location_type: LocationType) -> Self {
        self.location_type = location_type;
        self
    }

    pub fn coordinates(mut self, latitude: f64, longitude: f64) -> Self {
        self.latitude = latitude;
        self.longitude = longitude;
        self
    }

    pub fn address(mut self, address: impl Into<Option<String>>) -> Self {
        self.address = address.into();
        self
    }

    pub fn city(mut self, city: impl Into<Option<String>>) -> Self {
        self.city = city.into();
        self
    }

    pub fn postal_code(mut self, postal_code: impl Into<Option<String>>) -> Self {
        self.postal_code = postal_code.into();
        self
    }

    pub fn value(mut self, value: serde_json::Value) -> Self {
        self.extra_value = Some(value);
        self
    }

    pub async fn create(self, pool: &PgPool) -> EcoLocationStaging {
        let id = self.id.unwrap_or_else(uuid::Uuid::new_v4);

        let value = self.extra_value.unwrap_or_else(|| {
            json!({
                "coordinates": {
                    "latitude": self.latitude.to_string(),
                    "longitude": self.longitude.to_string(),
                },
                "address": self.address,
                "city": self.city,
                "postal_code": self.postal_code,
            })
        });

        sqlx::query(
            "INSERT INTO eco_movement.location (id, value, type, operator_id) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(value)
        .bind(self.location_type)
        .bind(self.operator_id)
        .execute(pool)
        .await
        .expect("could not insert eco_movement.location fixture");

        EcoLocationStaging {
            id,
            operator_id: self.operator_id,
        }
    }
}
