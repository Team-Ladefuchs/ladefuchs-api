// pub fn spawn_price_task(state: State) -> tokio::task::JoinHandle<()> {}

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use sqlx::PgConnection;

use crate::{
    eco_movement::db::{self},
    state::State,
};

use super::{
    api::{
        client::{EcoMovementClient, ResponseData, stream_all_data},
        response::{
            location::LocationData,
            price::{ConnectorPrice, PriceData},
        },
    },
    db::Table,
};

use futures_util::{pin_mut, stream::StreamExt};

pub async fn import_data(state: State) -> Result<(), eyre::ErrReport> {
    tracing::info!("import data");
    let eco_api = &state.eco_movement_api;

    let mut connection = state.database_pool.acquire().await?;

    // import(&mut connection, LocationImport { eco_api }).await?;
    // import(&mut connection, ConnectorPriceImport { eco_api }).await?;
    // import(&mut connection, PriceImport { eco_api }).await?;

    tracing::info!("import done");
    Ok(())
}

struct LocationImport<'a> {
    eco_api: &'a EcoMovementClient,
}

#[async_trait]
impl Importer<LocationData> for LocationImport<'_> {
    async fn fetch_page(
        &self,
        offset: usize,
    ) -> Result<ResponseData<LocationData>, reqwest::Error> {
        self.eco_api.fetch_location_page(offset).await
    }

    async fn save_multiple(
        connection: &mut PgConnection,
        locations: Vec<LocationData>,
    ) -> Result<(), sqlx::Error> {
        db::location::save_multiple(connection, &locations).await
    }

    fn table() -> Table {
        db::Table::Location
    }
}

struct ConnectorPriceImport<'a> {
    eco_api: &'a EcoMovementClient,
}

#[async_trait]
impl Importer<ConnectorPrice> for ConnectorPriceImport<'_> {
    async fn fetch_page(
        &self,
        offset: usize,
    ) -> Result<ResponseData<ConnectorPrice>, reqwest::Error> {
        self.eco_api.fetch_connector_prices_page(offset).await
    }

    async fn save_multiple(
        connection: &mut PgConnection,
        data: Vec<ConnectorPrice>,
    ) -> Result<(), sqlx::Error> {
        db::connector_prices::save_multiple(connection, data).await
    }

    fn table() -> Table {
        db::Table::ConnectorPrice
    }
}

struct PriceImport<'a> {
    eco_api: &'a EcoMovementClient,
}

#[async_trait]
impl Importer<PriceData> for PriceImport<'_> {
    async fn fetch_page(&self, offset: usize) -> Result<ResponseData<PriceData>, reqwest::Error> {
        self.eco_api.fetch_price_page(offset).await
    }

    async fn save_multiple(
        connection: &mut PgConnection,
        data: Vec<PriceData>,
    ) -> Result<(), sqlx::Error> {
        db::price::save_multiple(connection, &data).await
    }

    fn table() -> Table {
        db::Table::Price
    }
}

async fn import<T, ImporterImpl>(
    connection: &mut PgConnection,
    importer: ImporterImpl,
) -> Result<(), eyre::ErrReport>
where
    T: DeserializeOwned,
    ImporterImpl: Importer<T> + Send + Sync,
{
    ImporterImpl::truncate(connection).await?;

    let stream = stream_all_data(|offset| importer.fetch_page(offset));
    pin_mut!(stream);

    while let Some(data_result) = stream.next().await {
        let data = data_result?;
        ImporterImpl::save_multiple(connection, data).await?;
    }

    Ok(())
}

#[async_trait]
trait Importer<T>
where
    T: DeserializeOwned,
{
    fn table() -> db::Table;
    async fn truncate(connection: &mut PgConnection) -> Result<(), eyre::Error> {
        let query = format!("TRUNCATE TABLE eco_movement.{}", Self::table());
        sqlx::query(&query).execute(connection).await?;
        Ok(())
    }

    async fn fetch_page(&self, offset: usize) -> Result<ResponseData<T>, reqwest::Error>;
    async fn save_multiple(
        connection: &mut PgConnection,
        connector_prices: Vec<T>,
    ) -> Result<(), sqlx::Error>;
}
