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
    #[strum(to_string = "connector")]
    Connector,
    #[strum(to_string = "operator")]
    Operator,
    #[strum(to_string = "tariff")]
    Tariff,
}

pub mod location {

    use super::*;
    use crate::eco_movement::api::response::location::{LocationData, LocationType};
    use celes::Country;
    use uuid::uuid;

    pub async fn save_multiple(
        connection: &mut PgConnection,
        locations: &[LocationData],
    ) -> Result<(), sqlx::Error> {
        let mut transaction = connection.begin().await?;
        for location in locations
            .iter()
            // .filter(|item| item.country == Country::germany())
            .filter(|item| item.location_type != LocationType::Other)
        {
            match &location.operator {
                Some(operator) => {
                    connector::save_multiple(&mut transaction, &location.evses).await?;
                    let operator_id = operator::save(&mut transaction, operator).await?;
                    save(&mut transaction, location, &operator_id).await?;
                }
                None => {
                    tracing::debug!(
                        msg = "Location does not have an operator",
                        location_id = %location.id
                    );
                }
            }
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn location_exists(
        connection: &mut PgConnection,
        location_id: &uuid::Uuid,
    ) -> Option<uuid::Uuid> {
        sqlx::query_file_scalar!("sql/insert/eco_movement/location_by_id.sql", location_id)
            .fetch_optional(&mut *connection)
            .await
            .ok()
            .flatten()
    }

    async fn save(
        connection: &mut PgConnection,
        location: &LocationData,
        operator_id: &uuid::Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query_file!(
            "sql/insert/eco_movement/location.sql",
            location.id,
            location.value,
            location.location_type as _,
            operator_id,
        )
        .execute(&mut *connection)
        .await?;
        Ok(())
    }
}

mod connector {

    use super::*;

    use crate::eco_movement::api::response::location::{ConnectorType, Evse};

    async fn save(connection: &mut PgConnection, evse: &Evse) -> Result<(), sqlx::Error> {
        for connector in evse
            .connectors
            .iter()
            .filter(|item| item.connector_type != ConnectorType::Other)
        {
            sqlx::query_file!(
                "sql/insert/eco_movement/connector.sql",
                connector.id,
                evse.uid,
                connector.power_type as _,
                connector.max_power,
                connector.connector_type as _
            )
            .execute(&mut *connection)
            .await?;
        }

        Ok(())
    }

    pub type ConnectorKey<'a> = (&'a str, &'a str);

    pub async fn connector_exists<'a>(
        connection: &mut PgConnection,
        (connector_id, evse_id): ConnectorKey<'a>,
    ) -> Option<String> {
        sqlx::query_file_scalar!(
            "sql/insert/eco_movement/connector_by_id.sql",
            connector_id,
            evse_id
        )
        .fetch_optional(&mut *connection)
        .await
        .ok()
        .flatten()
    }

    pub async fn save_multiple(
        connection: &mut PgConnection,
        evses: &[Evse],
    ) -> Result<(), sqlx::Error> {
        let mut transaction = connection.begin().await?;
        for evse in evses {
            save(&mut transaction, evse).await?;
        }
        transaction.commit().await?;
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

    pub async fn price_exists(connection: &mut PgConnection, price_id: &str) -> Option<String> {
        sqlx::query_file_scalar!("sql/insert/eco_movement/price_by_id.sql", price_id)
            .fetch_optional(&mut *connection)
            .await
            .ok()
            .flatten()
    }

    async fn save(
        connection: &mut PgConnection,
        connector_price: ConnectorPrice,
    ) -> Result<(), sqlx::Error> {
        if let (Some(location_id), Some(connector_id)) = (
            location::location_exists(connection, &connector_price.location_id).await,
            connector::connector_exists(
                connection,
                (&connector_price.connector_id, &connector_price.evse_uid),
            )
            .await,
        ) {
            let mut price_ids = Vec::with_capacity(connector_price.pricing_ids.len());
            for p in connector_price.pricing_ids {
                if let Some(price_id) = price_exists(connection, &p).await {
                    price_ids.push(price_id);
                }
            }
            for price_id in price_ids {
                sqlx::query_file!(
                    "sql/insert/eco_movement/connector_price.sql",
                    location_id,
                    price_id,
                    connector_price.evse_uid,
                    connector_id
                )
                .execute(&mut *connection)
                .await?;
            }
        }
        Ok(())
    }
}

pub mod price {

    use super::*;
    use crate::eco_movement::api::response::price::{ComponentType, PriceData};

    pub async fn save_multiple(
        connection: &mut PgConnection,
        prices: &[PriceData],
    ) -> Result<(), sqlx::Error> {
        let mut transaction = connection.begin().await?;
        let filtered_prices = prices
            .iter()
            .filter(|item| item.tariff.currency == "EUR")
            .filter(|item| {
                item.elements.iter().all(|element| {
                    element.price_components.iter().all(|pc| {
                        pc.price_type != ComponentType::Time && pc.price_type != ComponentType::Flat
                    })
                })
            });
        for price in filtered_prices {
            let tariff_id =
                tariff::save(&mut transaction, &price.tariff, &price.provider_name).await?;
            save(&mut transaction, price, tariff_id).await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn save(
        connection: &mut PgConnection,
        price: &PriceData,
        tariff_id: i32,
    ) -> Result<(), sqlx::Error> {
        if let Ok(elements) = serde_json::to_value(&price.elements) {
            sqlx::query_file!(
                "sql/insert/eco_movement/price.sql",
                price.id,
                price.provider_name,
                tariff_id,
                elements,
            )
            .execute(&mut *connection)
            .await?;
        }

        Ok(())
    }
}

pub mod operator {

    use crate::eco_movement::api::response::operator::Operator;

    use super::*;

    pub async fn save(
        connection: &mut PgConnection,
        operator: &Operator,
    ) -> Result<uuid::Uuid, sqlx::Error> {
        sqlx::query_file!(
            "sql/insert/eco_movement/operator.sql",
            operator.id,
            operator.name,
            operator.website,
            &operator.ema_id
        )
        .execute(&mut *connection)
        .await?;
        Ok(operator.id)
    }
}

pub mod tariff {
    use crate::eco_movement::api::response::tariff::Tariff;

    use super::*;

    pub async fn save(
        connection: &mut PgConnection,
        tariff: &Tariff,
        provider_name: &str,
    ) -> Result<i32, sqlx::Error> {
        sqlx::query_file_scalar!(
            "sql/insert/eco_movement/tariff.sql",
            tariff.name,
            tariff.description,
            tariff.subscription_type,
            tariff._type as _,
            tariff.subscription_fee_excl_vat,
            tariff.currency,
            provider_name
        )
        .fetch_one(&mut *connection)
        .await
    }
}

pub async fn truncate(connection: &mut PgConnection, table: Table) -> Result<(), sqlx::Error> {
    let query = format!("TRUNCATE TABLE eco_movement.{} cascade", table);
    sqlx::query(&query).execute(connection).await?;
    Ok(())
}
