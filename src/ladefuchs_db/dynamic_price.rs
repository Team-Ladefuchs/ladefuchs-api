use chrono::{NaiveDate, NaiveTime};
use sqlx::PgConnection;

use super::plug::ChargeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, PartialOrd, Ord)]
#[sqlx(type_name = "day_of_week", rename_all = "lowercase")]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

pub const ALL_DAYS: [DayOfWeek; 7] = [
    DayOfWeek::Monday,
    DayOfWeek::Tuesday,
    DayOfWeek::Wednesday,
    DayOfWeek::Thursday,
    DayOfWeek::Friday,
    DayOfWeek::Saturday,
    DayOfWeek::Sunday,
];

pub fn weekday_to_day_of_week(day: chrono::Weekday) -> DayOfWeek {
    match day {
        chrono::Weekday::Mon => DayOfWeek::Monday,
        chrono::Weekday::Tue => DayOfWeek::Tuesday,
        chrono::Weekday::Wed => DayOfWeek::Wednesday,
        chrono::Weekday::Thu => DayOfWeek::Thursday,
        chrono::Weekday::Fri => DayOfWeek::Friday,
        chrono::Weekday::Sat => DayOfWeek::Saturday,
        chrono::Weekday::Sun => DayOfWeek::Sunday,
    }
}

pub fn eco_days_to_days_of_week(days: &[String]) -> Vec<DayOfWeek> {
    let mut result: Vec<DayOfWeek> = days
        .iter()
        .filter_map(|d| match d.to_uppercase().as_str() {
            "MONDAY" => Some(DayOfWeek::Monday),
            "TUESDAY" => Some(DayOfWeek::Tuesday),
            "WEDNESDAY" => Some(DayOfWeek::Wednesday),
            "THURSDAY" => Some(DayOfWeek::Thursday),
            "FRIDAY" => Some(DayOfWeek::Friday),
            "SATURDAY" => Some(DayOfWeek::Saturday),
            "SUNDAY" => Some(DayOfWeek::Sunday),
            _ => None,
        })
        .collect();

    result.sort();

    if result.is_empty() {
        ALL_DAYS.to_vec()
    } else {
        result
    }
}

#[derive(Debug)]
pub struct EcoLocation {
    pub eco_movement_id: uuid::Uuid,
    pub operator_id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
}

#[derive(Debug)]
pub struct EcoDynamicPrice {
    pub eco_location_id: uuid::Uuid,
    pub operator_id: i32,
    pub tariff_id: i32,
    pub price: f64,
    pub power_type: ChargeType,
    pub blocking_fee_start: Option<i32>,
    pub blocking_fee: Option<f64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub day_of_week_json: Option<serde_json::Value>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

pub async fn clear_all(transaction: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query(
        "TRUNCATE TABLE location_dynamic_price, dynamic_charge_price, charging_location CASCADE",
    )
    .execute(&mut *transaction)
    .await?;
    Ok(())
}

pub async fn save_locations(
    transaction: &mut PgConnection,
    locations: &[EcoLocation],
) -> Result<(), sqlx::Error> {
    for loc in locations {
        sqlx::query(
            "INSERT INTO charging_location (eco_movement_id, operator_id, geo, address, city, postal_code)
             VALUES ($1, $2, ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography, $5, $6, $7)
             ON CONFLICT (eco_movement_id) DO NOTHING"
        )
        .bind(loc.eco_movement_id)
        .bind(loc.operator_id)
        .bind(loc.longitude)
        .bind(loc.latitude)
        .bind(&loc.address)
        .bind(&loc.city)
        .bind(&loc.postal_code)
        .execute(&mut *transaction)
        .await?;
    }

    Ok(())
}

struct DynamicPriceRow {
    operator_id: i32,
    tariff_id: i32,
    c_type: ChargeType,
    price: f64,
    blocking_fee_start: i64,
    blocking_fee: f64,
    day_of_week: Vec<DayOfWeek>,
    start_time: Option<NaiveTime>,
    end_time: Option<NaiveTime>,
    valid_from: Option<NaiveDate>,
    valid_until: Option<NaiveDate>,
}

pub async fn save_dynamic_prices_and_mappings(
    transaction: &mut PgConnection,
    prices: &[EcoDynamicPrice],
) -> Result<(), sqlx::Error> {
    use std::collections::HashMap;

    let mut price_to_locations: HashMap<String, (DynamicPriceRow, Vec<uuid::Uuid>)> =
        HashMap::new();

    for p in prices {
        let dow = match &p.day_of_week_json {
            Some(val) => {
                if let Some(arr) = val.as_array() {
                    let days: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    eco_days_to_days_of_week(&days)
                } else {
                    ALL_DAYS.to_vec()
                }
            }
            None => ALL_DAYS.to_vec(),
        };

        let start_time = p
            .start_time
            .as_ref()
            .and_then(|s| NaiveTime::parse_from_str(s, "%H:%M").ok());
        let end_time = p
            .end_time
            .as_ref()
            .and_then(|s| NaiveTime::parse_from_str(s, "%H:%M").ok());

        let valid_from = p
            .start_date
            .as_ref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
        let valid_until = p
            .end_date
            .as_ref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

        let key = format!(
            "{}-{}-{:?}-{:?}-{:?}-{:?}-{:?}-{:?}",
            p.operator_id,
            p.tariff_id,
            p.power_type,
            dow,
            start_time,
            end_time,
            valid_from,
            valid_until
        );

        let entry = price_to_locations.entry(key).or_insert_with(|| {
            (
                DynamicPriceRow {
                    operator_id: p.operator_id,
                    tariff_id: p.tariff_id,
                    c_type: p.power_type,
                    price: p.price,
                    blocking_fee_start: p.blocking_fee_start.map(i64::from).unwrap_or_default(),
                    blocking_fee: p.blocking_fee.unwrap_or_default(),
                    day_of_week: dow,
                    start_time,
                    end_time,
                    valid_from,
                    valid_until,
                },
                Vec::new(),
            )
        });
        entry.1.push(p.eco_location_id);
    }

    let entries: Vec<_> = price_to_locations.into_values().collect();

    for chunk in entries.chunks(500) {
        for (price_row, location_ids) in chunk {
            let price_id: i32 = sqlx::query_scalar(
                "INSERT INTO dynamic_charge_price (operator_id, tariff_id, c_type, price, blocking_fee_start, blocking_fee, day_of_week, start_time, end_time, valid_from, valid_until, updated)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, now())
                 ON CONFLICT (operator_id, tariff_id, c_type, day_of_week, start_time, end_time, valid_from, valid_until)
                 DO UPDATE SET price = EXCLUDED.price, blocking_fee_start = EXCLUDED.blocking_fee_start, blocking_fee = EXCLUDED.blocking_fee, updated = now()
                 RETURNING id"
            )
            .bind(price_row.operator_id)
            .bind(price_row.tariff_id)
            .bind(price_row.c_type)
            .bind(price_row.price)
            .bind(price_row.blocking_fee_start)
            .bind(price_row.blocking_fee)
            .bind(&price_row.day_of_week)
            .bind(price_row.start_time)
            .bind(price_row.end_time)
            .bind(price_row.valid_from)
            .bind(price_row.valid_until)
            .fetch_one(&mut *transaction)
            .await?;

            if !location_ids.is_empty() {
                let unique_location_ids: Vec<&uuid::Uuid> = {
                    let mut seen = std::collections::HashSet::new();
                    location_ids.iter().filter(|id| seen.insert(*id)).collect()
                };

                for loc_chunk in unique_location_ids.chunks(500) {
                    let mut junction_builder = sqlx::QueryBuilder::new(
                        "INSERT INTO location_dynamic_price (location_id, dynamic_price_id)
                         SELECT cl.id, ",
                    );
                    junction_builder.push_bind(price_id);
                    junction_builder
                        .push(" FROM charging_location cl WHERE cl.eco_movement_id IN (");

                    let mut separated = junction_builder.separated(", ");
                    for loc_id in loc_chunk {
                        separated.push_bind(*loc_id);
                    }
                    separated.push_unseparated(") ON CONFLICT DO NOTHING");

                    junction_builder.build().execute(&mut *transaction).await?;
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
pub struct NearbyLocationPrice {
    pub location_id: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub city: Option<String>,
    pub distance: f64,
    pub tariff_id: uuid::Uuid,
    pub tariff_name: String,
    pub charging_mode: ChargeType,
    pub price_per_kwh: f64,
    pub blocking_fee_start: i64,
    pub blocking_fee: f64,
    pub valid_from: Option<NaiveDate>,
    pub valid_until: Option<NaiveDate>,
}

pub async fn find_nearby_with_prices(
    connection: &mut PgConnection,
    longitude: f64,
    latitude: f64,
    radius: f64,
    time: NaiveTime,
    day: DayOfWeek,
    date: NaiveDate,
) -> Result<Vec<NearbyLocationPrice>, sqlx::Error> {
    let rows = sqlx::query_file_as!(
        NearbyLocationPrice,
        "sql/get/dynamic_price/by_location.sql",
        longitude,
        latitude,
        radius,
        time,
        day as DayOfWeek,
        date
    )
    .fetch_all(&mut *connection)
    .await?;

    Ok(rows)
}
