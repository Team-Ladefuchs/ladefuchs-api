use crate::slack::{Slack, SlackClient};
use crate::{eco_movement, ladefuchs_db};
use sqlx::PgConnection;
use tracing::{info, warn};

pub async fn import(
    transaction: &mut PgConnection,
    slack: &Option<Slack>,
) -> Result<(), sqlx::Error> {
    info!("Importing charging locations");
    let locations = eco_movement::db::dynamic_price::get_locations(transaction).await?;

    info!(count = locations.len(), "Found charging locations");
    ladefuchs_db::dynamic_price::save_locations(transaction, &locations).await?;

    info!("Importing dynamic prices");
    let prices = eco_movement::db::dynamic_price::get_dynamic_prices(transaction).await?;

    info!(count = prices.len(), "Found dynamic prices");
    ladefuchs_db::dynamic_price::save_dynamic_prices_and_mappings(transaction, &prices).await?;

    if locations.is_empty() || prices.is_empty() {
        warn!(
            locations = locations.len(),
            prices = prices.len(),
            "Skipping stale sweep: dynamic feed empty, keeping existing rows"
        );

        slack
            .send_warning_message(
                "Dynamic-Preisimport: leerer Feed von Eco-Movement, Sweep übersprungen und Bestand behalten."
                    .to_string(),
            )
            .await;

        return Ok(());
    }

    info!("Sweeping stale dynamic-price rows");
    ladefuchs_db::dynamic_price::sweep_stale(transaction).await?;

    Ok(())
}
