use crate::{eco_movement, ladefuchs_db};
use sqlx::PgConnection;

pub async fn import(transaction: &mut PgConnection) -> Result<(), sqlx::Error> {
    let tariffs = eco_movement::db::tariff::get_all(transaction).await?;
    ladefuchs_db::tariff::add_or_update_tariffs(transaction, &tariffs).await?;

    Ok(())
}
