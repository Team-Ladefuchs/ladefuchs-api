use crate::eco_movement::api::response::location::LocationData;

use super::*;

pub struct LocationImport<'a> {
    pub eco_api: &'a EcoMovementClient,
}

#[async_trait]
impl EcoImport<LocationData> for LocationImport<'_> {
    async fn fetch_page(
        &self,
        offset: usize,
    ) -> Result<ResponseData<LocationData>, reqwest::Error> {
        self.eco_api.fetch_page(Endpoint::Location, offset).await
    }

    async fn save_multiple(
        connection: &mut PgConnection,
        locations: Vec<LocationData>,
    ) -> Result<(), sqlx::Error> {
        db::location::save_multiple(connection, &locations).await
    }

    async fn truncate(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
        truncate(connection, Table::Operator).await?;
        truncate(connection, Table::Location).await?;

        Ok(())
    }
}
