use crate::eco_movement::api::response::price::ConnectorPrice;

use super::*;

pub struct ConnectorPriceImport<'a> {
    pub eco_api: &'a EcoMovementClient,
}

#[async_trait]
impl EcoImport<ConnectorPrice> for ConnectorPriceImport<'_> {
    async fn fetch_page(
        &self,
        offset: usize,
    ) -> Result<ResponseData<ConnectorPrice>, reqwest::Error> {
        self.eco_api
            .fetch_page(Endpoint::ConnectorPrice, offset)
            .await
    }

    async fn save_multiple(
        connection: &mut PgConnection,
        data: Vec<ConnectorPrice>,
    ) -> Result<(), sqlx::Error> {
        db::connector_prices::save_multiple(connection, data).await
    }

    async fn truncate(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
        truncate(connection, Table::ConnectorPrice).await
    }
}
