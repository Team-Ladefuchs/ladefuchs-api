use super::*;
use crate::eco_movement::api::response::location::{LocationData, LocationType, RestrictionType};

pub async fn save_multiple(
    connection: &mut PgConnection,
    locations: &[LocationData],
) -> Result<(), sqlx::Error> {
    let mut transaction = connection.begin().await?;

    let locations_iter = locations
        .iter()
        .filter(|item| item.country == "DEU")
        .filter(|item| {
            item.restrictions
                .as_ref()
                .map(|restrictions| {
                    restrictions.is_empty()
                        || restrictions.iter().all(|r| {
                            matches!(
                                r,
                                RestrictionType::Customers | RestrictionType::TimeRestricted
                            )
                        })
                })
                .unwrap_or(true) // Allow if restrictions is None
        })
        .filter(|item| item.location_type != LocationType::Other);

    for location in locations_iter {
        match &location.operator {
            Some(operator) => {
                connector::save_multiple(&mut transaction, &location.evses).await?;
                let operator_id = operator::save(&mut transaction, operator).await?;
                save(&mut transaction, location, &operator_id).await?;
            }

            None => {
                tracing::debug!(
                    msg = "Location does not have an operator",
                    location_id = %location.id
                );
            }
        }
    }

    transaction.commit().await?;

    Ok(())
}

pub async fn location_exists(
    connection: &mut PgConnection,
    location_id: &uuid::Uuid,
) -> Option<uuid::Uuid> {
    sqlx::query_file_scalar!("sql/insert/eco_movement/location_by_id.sql", location_id)
        .fetch_optional(&mut *connection)
        .await
        .ok()
        .flatten()
}

async fn save(
    connection: &mut PgConnection,
    location: &LocationData,
    operator_id: &uuid::Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "sql/insert/eco_movement/location.sql",
        location.id,
        location.value,
        location.location_type as _,
        operator_id,
    )
    .execute(&mut *connection)
    .await?;

    Ok(())
}
