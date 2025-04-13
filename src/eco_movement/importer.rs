// pub fn spawn_price_task(state: State) -> tokio::task::JoinHandle<()> {}

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use sqlx::PgConnection;

use crate::eco_movement::api::client::Endpoint;
use crate::{
    eco_movement::db::{self},
    state::State,
};

use super::{
    api::client::{EcoMovementClient, ResponseData, stream_all_data},
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
    import(&mut connection, tariff::TariffImport { eco_api }).await?;

    tracing::info!("import done");
    Ok(())
}

pub mod location {
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

        fn table() -> Table {
            db::Table::Location
        }
    }
}

mod connector_price {
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

        fn table() -> Table {
            db::Table::ConnectorPrice
        }
    }
}

mod price {
    use crate::eco_movement::api::response::price::PriceData;

    use super::*;
    pub struct PriceImport<'a> {
        pub eco_api: &'a EcoMovementClient,
    }

    #[async_trait]
    impl EcoImport<PriceData> for PriceImport<'_> {
        async fn fetch_page(
            &self,
            offset: usize,
        ) -> Result<ResponseData<PriceData>, reqwest::Error> {
            self.eco_api.fetch_page(Endpoint::Price, offset).await
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
}

mod tariff {
    use crate::eco_movement::api::response::tariff::TariffData;

    use super::*;

    pub struct TariffImport<'a> {
        pub eco_api: &'a EcoMovementClient,
    }

    #[async_trait]
    impl EcoImport<TariffData> for TariffImport<'_> {
        async fn fetch_page(
            &self,
            offset: usize,
        ) -> Result<ResponseData<TariffData>, reqwest::Error> {
            self.eco_api.fetch_page(Endpoint::Tariff, offset).await
        }

        async fn save_multiple(
            connection: &mut PgConnection,
            data: Vec<TariffData>,
        ) -> Result<(), sqlx::Error> {
            db::tariff::save_multiple(connection, &data).await
        }

        fn table() -> Table {
            db::Table::Tariff
        }
    }
}

async fn import<T, ImporterImpl>(
    connection: &mut PgConnection,
    importer: ImporterImpl,
) -> Result<(), eyre::ErrReport>
where
    T: DeserializeOwned,
    ImporterImpl: EcoImport<T> + Send + Sync,
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
trait EcoImport<T>
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
