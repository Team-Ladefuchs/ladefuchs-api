use crate::{
    eco_movement::{self, api::response::price::PriceData},
    ladefuchs_db,
};

use super::*;

pub struct PriceImport<'a> {
    pub eco_api: &'a EcoMovementClient,
}

#[async_trait]
impl EcoImport<PriceData> for PriceImport<'_> {
    async fn fetch_page(&self, offset: usize) -> Result<ResponseData<PriceData>, reqwest::Error> {
        self.eco_api.fetch_page(Endpoint::Price, offset).await
    }

    async fn save_multiple(
        connection: &mut PgConnection,
        data: Vec<PriceData>,
    ) -> Result<(), sqlx::Error> {
        db::price::save_multiple(connection, data).await
    }

    async fn truncate(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
        truncate(connection, Table::Tariff).await?;
        truncate(connection, Table::Price).await?;

        Ok(())
    }
}

pub async fn import(transaction: &mut PgConnection) -> Result<usize, sqlx::Error> {
    let prices = eco_movement::db::price::get_all(transaction).await?;

    ladefuchs_db::price::clear_all(transaction).await?;
    ladefuchs_db::price::save_all(transaction, &prices).await?;

    Ok(prices.len())
}
