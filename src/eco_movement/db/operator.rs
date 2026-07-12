use crate::eco_movement::api::response::operator::Operator;

use super::*;

pub async fn save(
    connection: &mut PgConnection,
    operator: &Operator,
) -> Result<uuid::Uuid, sqlx::Error> {
    sqlx::query_file!(
        "sql/insert/eco_movement/operator.sql",
        operator.id,
        operator.name,
        operator.website,
        &operator.ema_id
    )
    .execute(&mut *connection)
    .await?;

    Ok(operator.id)
}

pub async fn get_standard_with_no_prices(
    connection: &mut PgConnection,
) -> Result<Vec<String>, sqlx::Error> {
    let operators_names =
        sqlx::query_file_scalar!("sql/get/operator/import/inactive_operators.sql")
            .fetch_all(&mut *connection)
            .await?;

    Ok(operators_names)
}

pub async fn get_all(connection: &mut PgConnection) -> Result<Vec<Operator>, sqlx::Error> {
    sqlx::query_file_as!(Operator, "sql/get/eco_movement/all_operator.sql")
        .fetch_all(&mut *connection)
        .await
}
