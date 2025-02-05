use sqlx::PgConnection;

use crate::charge_price_api::request::feedback::Feedback;

pub async fn save(
    transaction: &mut PgConnection,
    feedback: Feedback,
) -> Result<(), sqlx::error::Error> {
    sqlx::query_file!(
        "sql/insert/feedback/add_feedback.sql",
        feedback.operator_id,
        feedback.tariff_id,
        feedback.language.to_string(),
        feedback.notes,
        feedback.kind as _,
        serde_json::to_value(&feedback.context).ok()
    )
    .execute(transaction)
    .await?;

    Ok(())
}
