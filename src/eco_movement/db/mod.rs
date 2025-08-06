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
    #[strum(to_string = "operator")]
    Operator,
    #[strum(to_string = "tariff")]
    Tariff,
}

pub mod location {

    use super::*;
    use crate::eco_movement::api::response::location::{
        LocationData, LocationType, RestrictionType,
    };
    // use celes::Country;
    // use uuid::uuid;

    pub async fn save_multiple(
        connection: &mut PgConnection,
        locations: &[LocationData],
    ) -> Result<(), sqlx::Error> {
        let mut transaction = connection.begin().await?;
        for location in locations
            .iter()
            .filter(|item| item.country == "DEU")
            .filter(|item| {
                item.restrictions
                    .as_ref()
                    .map(|restrictions| {
                        restrictions.is_empty()
                            || restrictions
                                .iter()
                                .all(|r| r == &RestrictionType::Customers)
                    })
                    .unwrap_or(true) // Allow if restrictions is None
            })
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

    #[derive(Debug)]
    struct PriceContext<'a> {
        location_id: uuid::Uuid,
        pricing_id: String,
        evse_uid: &'a str,
        connector_id: &'a str,
    }

    pub async fn save_multiple(
        connection: &mut PgConnection,
        connector_prices: Vec<ConnectorPrice>,
    ) -> Result<(), sqlx::Error> {
        for connector_price in connector_prices {
            save(connection, connector_price).await?;
        }

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
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO eco_movement.connector_price (location_id, pricing_id, evse_uid, connector_id)",
            );
            let mut price_queries = Vec::with_capacity(connector_price.pricing_ids.len());
            tracing::debug!("build price query start");
            for pricing_id in connector_price.pricing_ids {
                if let Some(price_id) = price_exists(connection, &pricing_id).await {
                    price_queries.push(PriceContext {
                        location_id,
                        evse_uid: &connector_price.evse_uid,
                        pricing_id: price_id,
                        connector_id: &connector_id,
                    });
                }
            }
            tracing::debug!(len = price_queries.len(), "build price query done");

            if price_queries.is_empty() {
                return Ok(());
            }

            query_builder.push_values(price_queries, |mut builder, new_price| {
                builder
                    .push_bind(new_price.location_id)
                    .push_bind(new_price.pricing_id)
                    .push_bind(new_price.evse_uid)
                    .push_bind(new_price.connector_id);
            });
            if let Err(error) = query_builder.build().execute(connection).await {
                tracing::error!(error=?error, "Possible duplicate ids again..");
            };
            tracing::debug!("insert price done");
        }
        Ok(())
    }
}

pub mod price {

    use super::*;
    use crate::{
        eco_movement::api::response::price::{ComponentType, PriceData},
        ladefuchs_db::plug::ChargeType,
    };

    pub async fn save_multiple(
        connection: &mut PgConnection,
        prices: Vec<PriceData>,
    ) -> Result<(), sqlx::Error> {
        let mut filtered_prices = prices
            .into_iter()
            .filter(|item| item.tariff.currency == "EUR")
            .filter(|item| {
                item.elements.iter().all(|element| {
                    element
                        .price_components
                        .iter()
                        .all(|pc| pc.price_type != ComponentType::Flat)
                })
            })
            .collect::<Vec<PriceData>>();

        if filtered_prices.len() == 1
            && filtered_prices
                .first()
                .and_then(|a| a.elements.first())
                .and_then(|dd| dd.price_components.first())
                .map_or(false, |pp| pp.price_type == ComponentType::ParkingTime)
        {
            filtered_prices.clear();
        }

        for price in &mut filtered_prices {
            let tariff_id = tariff::save(connection, &price.tariff, &price.provider_name).await?;
            for element in &mut price.elements {
                for comp in &mut element.price_components {
                    if comp.price_type == ComponentType::ParkingTime && comp.price_excl_vat > 0.95 {
                        comp.price_excl_vat /= 60.0;
                    }
                }
                if let Some(restrictions) = &mut element.restrictions {
                    if let Some(min_duration) = restrictions.min_duration {
                        if min_duration > 900 {
                            restrictions.min_duration = Some(min_duration / 60);
                        }
                    }
                }
            }
            save(connection, &price, &tariff_id).await?;
        }
        Ok(())
    }

    async fn save(
        connection: &mut PgConnection,
        price: &PriceData,
        tariff_id: &uuid::Uuid,
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

    #[derive(Debug)]
    pub struct EcoPrice {
        pub tariff_id: i32,
        pub operator_id: i32,
        pub power_type: ChargeType,
        pub price_kw: f64,
        pub blocking_fee_start: Option<i32>,
        pub blocking_fee: Option<f64>,
    }

    pub async fn get_all(connection: &mut PgConnection) -> Result<Vec<EcoPrice>, sqlx::Error> {
        sqlx::query_file_as!(
            EcoPrice,
            "sql/get/eco_movement/get_price_tariff_operator.sql"
        )
        .fetch_all(&mut *connection)
        .await
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

    pub async fn get_standard_with_no_prices(
        connection: &mut PgConnection,
    ) -> Result<Vec<String>, sqlx::Error> {
        let operators_names =
            sqlx::query_file_scalar!("sql/get/operator/import/inactive_operators.sql")
                .fetch_all(&mut *connection)
                .await?;
        Ok(operators_names)
    }

    pub async fn get_all(connection: &mut PgConnection) -> Result<Vec<Operator>, sqlx::Error> {
        sqlx::query_file_as!(Operator, "sql/get/eco_movement/all_operator.sql")
            .fetch_all(&mut *connection)
            .await
    }
}

pub mod tariff {
    use crate::{
        eco_movement::api::response::tariff::{Tariff, TariffType},
        ladefuchs_db::tariff::CUSTOMER_ONLY_TARIFFS_NAME,
    };

    use super::*;

    #[derive(Debug)]
    pub struct EcoTariff {
        pub network: uuid::Uuid,
        pub name: String,
        pub description: Option<String>,
        pub tariff_type: TariffType,
        pub provider_name: String,
        pub subscription_fee: Option<f64>,
    }

    impl EcoTariff {
        pub fn is_ad_hoc(&self) -> bool {
            self.tariff_type == TariffType::Adhoc
        }
        pub fn is_standard(&self) -> bool {
            self.subscription_fee <= Some(0.0) && !self.is_customer_only()
        }
        pub fn is_customer_only(&self) -> bool {
            if let Some(desc) = &self.description {
                if CUSTOMER_ONLY_TARIFFS_NAME.is_match(desc) {
                    return true;
                }
            }
            CUSTOMER_ONLY_TARIFFS_NAME.is_match(&self.name)
                || CUSTOMER_ONLY_TARIFFS_NAME.is_match(&self.provider_name)
        }
    }

    pub async fn save(
        connection: &mut PgConnection,
        tariff: &Tariff,
        provider_name: &str,
    ) -> Result<uuid::Uuid, sqlx::Error> {
        let id = uuid::Uuid::now_v7();
        sqlx::query_file_scalar!(
            "sql/insert/eco_movement/tariff.sql",
            tariff.name,
            tariff.description,
            tariff.subscription_type,
            tariff._type as _,
            tariff.subscription_fee_excl_vat,
            tariff.currency,
            provider_name,
            id,
        )
        .fetch_one(&mut *connection)
        .await
    }

    pub async fn get_all(connection: &mut PgConnection) -> Result<Vec<EcoTariff>, sqlx::Error> {
        sqlx::query_file_as!(EcoTariff, "sql/get/eco_movement/all_tariff.sql")
            .fetch_all(&mut *connection)
            .await
    }
}

pub async fn truncate(connection: &mut PgConnection, table: Table) -> Result<(), sqlx::Error> {
    let query = format!("TRUNCATE TABLE eco_movement.{} cascade", table);
    sqlx::query(&query).execute(connection).await?;
    Ok(())
}
