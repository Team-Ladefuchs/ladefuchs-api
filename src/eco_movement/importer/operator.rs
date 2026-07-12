use crate::{eco_movement, ladefuchs_db};
use sqlx::PgConnection;

pub async fn import(transaction: &mut PgConnection) -> Result<(), eyre::Error> {
    let operators = eco_movement::db::operator::get_all(transaction).await?;
    ladefuchs_db::operator::insert_or_update_operators(transaction, &operators).await?;

    Ok(())
}
