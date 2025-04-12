// pub fn spawn_price_task(state: State) -> tokio::task::JoinHandle<()> {}

use eyre::Ok;
use sqlx::PgConnection;

use crate::{
    eco_movement::{
        api::client::{LIMIT_OFFSET_PAGE, MAX_PER_PAGE},
        db,
    },
    state::State,
};

use super::{api::client::EcoMovementClient, db::location::truncate};

pub async fn import_data(state: State) -> Result<(), eyre::ErrReport> {
    dbg!("import data");
    let eco_api = &state.eco_movement_api;

    let mut connection = state.database_pool.acquire().await?;

    import_locations(&mut connection, eco_api).await?;

    dbg!("import done");
    Ok(())
}

async fn import_locations(
    connection: &mut PgConnection,
    eco_api: &EcoMovementClient,
) -> Result<(), eyre::ErrReport> {
    truncate(connection).await?;

    let mut offset = 0;
    loop {
        let response = eco_api.fetch_location_page(offset).await?;
        tracing::info!(
            source = "fetch_all_locations",
            offset,
            locations = response.data.len()
        );
        offset += LIMIT_OFFSET_PAGE;

        db::location::save_multiple(connection, &response.data).await?;
        if response.data.len() < LIMIT_OFFSET_PAGE {
            return Ok(());
        }
        if offset > MAX_PER_PAGE {
            return Ok(());
        }
    }
}
