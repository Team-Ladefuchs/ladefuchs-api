use sqlx::Connection;
use sqlx::PgConnection;

#[derive(Debug, strum_macros::Display)]
pub enum Table {
    #[strum(to_string = "location")]
    Location,
    #[strum(to_string = "connector_price")]
    ConnectorPrice,
    #[strum(to_string = "price")]
    Price,
    #[strum(to_string = "tariff")]
    Tariff,
}

pub mod location {

    use super::*;
    use crate::eco_movement::api::response::location::LocationData;

    pub async fn save_multiple(
        connection: &mut PgConnection,
        locations: &[LocationData],
    ) -> Result<(), sqlx::Error> {
        let mut transaction = connection.begin().await?;
        for location in locations {
            save(&mut transaction, location).await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn save(
        connection: &mut PgConnection,
        location: &LocationData,
    ) -> Result<(), sqlx::Error> {
        sqlx::query_file!(
            "sql/insert/eco_movement/location.sql",
            location.id,
            location.value
        )
        .execute(&mut *connection)
        .await?;
        Ok(())
    }
}

pub mod connector_prices {
    use super::*;
    use crate::eco_movement::api::response::price::ConnectorPrice;

    pub async fn save_multiple(
        connection: &mut PgConnection,
        connector_prices: Vec<ConnectorPrice>,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = connection.begin().await?;
        for connector_price in connector_prices {
            save(&mut transaction, connector_price).await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn save(
        connection: &mut PgConnection,
        connector_price: ConnectorPrice,
    ) -> Result<(), sqlx::Error> {
        for price_id in connector_price.pricing_ids {
            sqlx::query_file!(
                "sql/insert/eco_movement/connector_price.sql",
                connector_price.location_id,
                price_id,
                connector_price.evse_uid,
                connector_price.connector_id
            )
            .execute(&mut *connection)
            .await?;
        }

        Ok(())
    }
}

pub mod price {
    use super::*;
    use crate::eco_movement::api::response::price::PriceData;

    pub async fn save_multiple(
        connection: &mut PgConnection,
        prices: &[PriceData],
    ) -> Result<(), sqlx::Error> {
        let mut transaction = connection.begin().await?;
        for price in prices {
            save(&mut transaction, price).await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn save(connection: &mut PgConnection, price: &PriceData) -> Result<(), sqlx::Error> {
        sqlx::query_file!("sql/insert/eco_movement/price.sql", price.id, price.value)
            .execute(&mut *connection)
            .await?;
        Ok(())
    }
}

pub mod tariff {
    use super::*;
    use crate::eco_movement::api::response::tariff::TariffData;

    pub async fn save_multiple(
        connection: &mut PgConnection,
        tariffs: &[TariffData],
    ) -> Result<(), sqlx::Error> {
        let mut transaction = connection.begin().await?;
        for price in tariffs {
            save(&mut transaction, price).await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn save(connection: &mut PgConnection, tariff: &TariffData) -> Result<(), sqlx::Error> {
        sqlx::query_file!(
            "sql/insert/eco_movement/tariff.sql",
            tariff.id,
            tariff.value
        )
        .execute(&mut *connection)
        .await?;
        Ok(())
    }
}
