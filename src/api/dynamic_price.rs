use axum::{Extension, Json, extract::rejection::JsonRejection};
use chrono::Datelike;
use serde::{Deserialize, Serialize};

use crate::{ladefuchs_db, state::State};

use super::{ApiJson, json};

const DEFAULT_RADIUS: f64 = 50.0;

pub mod v3 {
    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LocationConditionRequest {
        pub latitude: f64,
        pub longitude: f64,
        pub timestamp: chrono::DateTime<chrono::Utc>,
        pub radius: Option<f64>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LocationConditionResponse {
        pub locations: Vec<LocationEntry>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LocationEntry {
        pub latitude: f64,
        pub longitude: f64,
        pub address: Option<String>,
        pub city: Option<String>,
        pub distance: f64,
        pub operator_id: uuid::Uuid,
        pub operator_name: String,
        pub charging_conditions: Vec<ChargingCondition>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ChargingCondition {
        pub provider_id: uuid::Uuid,
        pub provider_name: String,
        pub tariff_id: uuid::Uuid,
        pub tariff_name: String,
        pub charging_mode: ladefuchs_db::plug::ChargeType,
        pub price_per_kwh: f64,
        pub blocking_fee_start: i64,
        pub blocking_fee: f64,
        pub valid_from: Option<chrono::NaiveDate>,
        pub valid_until: Option<chrono::NaiveDate>,
    }

    pub async fn post_handler(
        Extension(state): Extension<State>,
        body: Result<Json<LocationConditionRequest>, JsonRejection>,
    ) -> ApiJson<LocationConditionResponse> {
        let Json(request) = body?;

        let radius = request.radius.unwrap_or(DEFAULT_RADIUS);
        let naive_time = request.timestamp.time();
        let naive_date = request.timestamp.date_naive();
        let day = ladefuchs_db::dynamic_price::weekday_to_day_of_week(request.timestamp.weekday());

        let rows = ladefuchs_db::dynamic_price::find_nearby_with_prices(
            &mut *state.database_pool.acquire().await?,
            request.longitude,
            request.latitude,
            radius,
            naive_time,
            day,
            naive_date,
        )
        .await?;

        let mut locations: Vec<LocationEntry> = Vec::new();
        let mut current_location_id: Option<i64> = None;

        for row in &rows {
            if current_location_id != Some(row.location_id) {
                locations.push(LocationEntry {
                    latitude: row.latitude,
                    longitude: row.longitude,
                    address: row.address.clone(),
                    city: row.city.clone(),
                    distance: row.distance,
                    operator_id: row.cpo_id,
                    operator_name: row.cpo_name.clone(),
                    charging_conditions: Vec::new(),
                });

                current_location_id = Some(row.location_id);
            }

            if let Some(loc) = locations.last_mut() {
                loc.charging_conditions.push(ChargingCondition {
                    provider_id: row.provider_id,
                    provider_name: row.provider_name.clone(),
                    tariff_id: row.tariff_id,
                    tariff_name: row.tariff_name.clone(),
                    charging_mode: row.charging_mode,
                    price_per_kwh: row.price_per_kwh,
                    blocking_fee_start: row.blocking_fee_start,
                    blocking_fee: row.blocking_fee,
                    valid_from: row.valid_from,
                    valid_until: row.valid_until,
                });
            }
        }

        json(LocationConditionResponse { locations })
    }
}
