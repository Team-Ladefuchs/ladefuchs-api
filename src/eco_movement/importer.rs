// pub fn spawn_price_task(state: State) -> tokio::task::JoinHandle<()> {}

use std::fmt::Debug;

use crate::eco_movement::api::client::Endpoint;
use crate::{
    eco_movement::db::{self},
    state::State,
};
use async_trait::async_trait;
use db::truncate;
use serde::de::DeserializeOwned;
use sqlx::PgConnection;

use super::{
    api::client::{EcoMovementClient, ResponseData, stream_all_data},
    db::Table,
};

use futures_util::{pin_mut, stream::StreamExt};

pub async fn import_data(state: State) -> Result<(), eyre::ErrReport> {
    tracing::info!("import data");
    let eco_api = &state.eco_movement_api;

    let mut connection: sqlx::pool::PoolConnection<sqlx::Postgres> =
        state.database_pool.acquire().await?;

    // import(&mut connection, location::LocationImport { eco_api }).await?;
    // import(&mut connection, price::PriceImport { eco_api }).await?;
    // import(
    //     &mut connection,
    //     connector_price::ConnectorPriceImport { eco_api },
    // )
    // .await?;

    // operator::import_operator(&mut connection).await?;
    tariff::import_tariff(&mut connection).await?;

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

        async fn truncate(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
            truncate(connection, Table::Operator).await?;
            truncate(connection, Table::Location).await?;
            Ok(())
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
            let a = self
                .eco_api
                .fetch_page(Endpoint::ConnectorPrice, offset)
                .await;
            a
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
        async fn truncate(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
            truncate(connection, Table::Tariff).await?;
            truncate(connection, Table::Price).await?;
            Ok(())
        }
    }
}

async fn import<T, ImporterImpl>(
    connection: &mut PgConnection,
    importer: ImporterImpl,
) -> Result<(), eyre::ErrReport>
where
    T: DeserializeOwned + Debug,
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
    async fn truncate(connection: &mut PgConnection) -> Result<(), sqlx::Error>;
    async fn fetch_page(&self, offset: usize) -> Result<ResponseData<T>, reqwest::Error>;
    async fn save_multiple(
        connection: &mut PgConnection,
        connector_prices: Vec<T>,
    ) -> Result<(), sqlx::Error>;
}

pub mod operator {

    use crate::{eco_movement, ladefuchs_db};
    use sqlx::{Connection, PgConnection};
    pub async fn import_operator(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
        let mut transaction = connection.begin().await?;

        let operators = eco_movement::db::operator::get_all(&mut transaction).await?;

        ladefuchs_db::operator::insert_or_update_operators(&mut transaction, &operators).await?;

        transaction.commit().await?;

        Ok(())
    }
}

pub mod tariff {

    use crate::{eco_movement, ladefuchs_db};
    use sqlx::{Connection, PgConnection};

    pub async fn import_tariff(connection: &mut PgConnection) -> Result<(), sqlx::Error> {
        let mut transaction = connection.begin().await?;
        let tariffs = eco_movement::db::tariff::get_all(&mut transaction).await?;
        ladefuchs_db::tariff::add_or_update_tariffs(&mut transaction, &tariffs).await?;
        transaction.commit().await?;
        Ok(())
    }
}
