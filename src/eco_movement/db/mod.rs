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
                            || restrictions.iter().all(|r| {
                                matches!(
                                    r,
                                    RestrictionType::Customers | RestrictionType::TimeRestricted
                                )
                            })
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

    async fn connector_price_exists(
        connection: &mut PgConnection,
        context: &PriceContext<'_>,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_file_scalar!(
            "sql/get/eco_movement/connector_price_exsists.sql",
            context.location_id,
            context.pricing_id,
            context.evse_uid,
            context.connector_id
        )
        .fetch_one(&mut *connection)
        .await
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
                    let price_context = PriceContext {
                        location_id,
                        evse_uid: &connector_price.evse_uid,
                        pricing_id: price_id,
                        connector_id: &connector_id,
                    };
                    if connector_price_exists(connection, &price_context).await? {
                        continue;
                    }
                    price_queries.push(price_context);
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

            query_builder.build().execute(connection).await?;
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
        let mut filtered_prices: Vec<(PriceData, uuid::Uuid)> = prices
            .into_iter()
            .filter_map(|item| is_importable(&item).map(|product_id| (item, product_id)))
            .collect();

        drop_parking_only_singleton(&mut filtered_prices);

        for (price, product_id) in &mut filtered_prices {
            let tariff_id =
                tariff::save(connection, &price.tariff, &price.provider_name, *product_id).await?;

            normalize_durations(price);

            save(connection, price, &tariff_id).await?;
        }

        Ok(())
    }

    fn is_importable(price: &PriceData) -> Option<uuid::Uuid> {
        let Some(product_id) = price.tariff.id else {
            tracing::warn!(price_id = %price.id, "skipping price without product.id");
            return None;
        };

        if price.tariff.currency != "EUR" {
            return None;
        }

        let has_flat = price.elements.iter().any(|element| {
            element
                .price_components
                .iter()
                .any(|pc| pc.price_type == ComponentType::Flat)
        });

        if has_flat {
            return None;
        }

        Some(product_id)
    }

    fn drop_parking_only_singleton(filtered: &mut Vec<(PriceData, uuid::Uuid)>) {
        if filtered.len() == 1
            && filtered
                .first()
                .and_then(|(a, _)| a.elements.first())
                .and_then(|el| el.price_components.first())
                .is_some_and(|pc| pc.price_type == ComponentType::ParkingTime)
        {
            filtered.clear();
        }
    }

    fn normalize_durations(price: &mut PriceData) {
        for element in &mut price.elements {
            for comp in &mut element.price_components {
                if comp.price_type == ComponentType::ParkingTime && comp.price_excl_vat > 0.95 {
                    comp.price_excl_vat /= 60.0;
                }
            }

            if let Some(restrictions) = &mut element.restrictions
                && let Some(min_duration) = restrictions.min_duration
                && min_duration > 900
            {
                restrictions.min_duration = Some(min_duration / 60);
            }
        }
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
        pub product_id: Option<uuid::Uuid>,
    }

    pub async fn get_all(connection: &mut PgConnection) -> Result<Vec<EcoPrice>, sqlx::Error> {
        sqlx::query_file_as!(
            EcoPrice,
            "sql/get/eco_movement/get_price_tariff_operator.sql"
        )
        .fetch_all(&mut *connection)
        .await
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::eco_movement::api::response::{
            price::{Components, Elements, Restrictions},
            tariff::{Tariff, TariffType},
        };

        fn tariff(currency: &str, id: Option<uuid::Uuid>) -> Tariff {
            Tariff {
                id,
                name: "Test".to_string(),
                description: String::new(),
                subscription_type: String::new(),
                subscription_fee_excl_vat: "0".to_string(),
                _type: TariffType::Msp,
                currency: currency.to_string(),
            }
        }

        fn component(price_type: ComponentType, price_excl_vat: f64) -> Components {
            Components {
                price_excl_vat,
                vat: 19,
                step_size: 1,
                price_type,
            }
        }

        fn element(components: Vec<Components>, restrictions: Option<Restrictions>) -> Elements {
            Elements {
                price_components: components,
                restrictions,
            }
        }

        fn price(currency: &str, id: Option<uuid::Uuid>, elements: Vec<Elements>) -> PriceData {
            PriceData {
                id: "p1".to_string(),
                provider_name: "Provider".to_string(),
                tariff: tariff(currency, id),
                elements,
            }
        }

        #[test]
        fn is_importable_keeps_eur_with_product_id_and_no_flat() {
            let id = uuid::Uuid::new_v4();
            let p = price(
                "EUR",
                Some(id),
                vec![element(vec![component(ComponentType::Energy, 0.5)], None)],
            );

            assert_eq!(is_importable(&p), Some(id));
        }

        #[test]
        fn is_importable_drops_missing_product_id() {
            let p = price(
                "EUR",
                None,
                vec![element(vec![component(ComponentType::Energy, 0.5)], None)],
            );

            assert_eq!(is_importable(&p), None);
        }

        #[test]
        fn is_importable_drops_non_eur_currency() {
            let p = price(
                "USD",
                Some(uuid::Uuid::new_v4()),
                vec![element(vec![component(ComponentType::Energy, 0.5)], None)],
            );

            assert_eq!(is_importable(&p), None);
        }

        #[test]
        fn is_importable_drops_when_any_element_has_flat_component() {
            let p = price(
                "EUR",
                Some(uuid::Uuid::new_v4()),
                vec![
                    element(vec![component(ComponentType::Energy, 0.5)], None),
                    element(vec![component(ComponentType::Flat, 1.0)], None),
                ],
            );
            assert_eq!(is_importable(&p), None);
        }

        #[test]
        fn drop_parking_only_singleton_clears_when_single_parking_only() {
            let id = uuid::Uuid::new_v4();

            let p = price(
                "EUR",
                Some(id),
                vec![element(
                    vec![component(ComponentType::ParkingTime, 0.1)],
                    None,
                )],
            );

            let mut v = vec![(p, id)];
            drop_parking_only_singleton(&mut v);

            assert!(v.is_empty());
        }

        #[test]
        fn drop_parking_only_singleton_keeps_when_multiple() {
            let id = uuid::Uuid::new_v4();

            let p1 = price(
                "EUR",
                Some(id),
                vec![element(
                    vec![component(ComponentType::ParkingTime, 0.1)],
                    None,
                )],
            );

            let p2 = price(
                "EUR",
                Some(id),
                vec![element(
                    vec![component(ComponentType::ParkingTime, 0.1)],
                    None,
                )],
            );

            let mut v = vec![(p1, id), (p2, id)];
            drop_parking_only_singleton(&mut v);

            assert_eq!(v.len(), 2);
        }

        #[test]
        fn drop_parking_only_singleton_keeps_when_first_component_is_energy() {
            let id = uuid::Uuid::new_v4();

            let p = price(
                "EUR",
                Some(id),
                vec![element(vec![component(ComponentType::Energy, 0.5)], None)],
            );

            let mut v = vec![(p, id)];
            drop_parking_only_singleton(&mut v);

            assert_eq!(v.len(), 1);
        }

        #[test]
        fn normalize_durations_divides_parking_time_when_above_threshold() {
            let mut p = price(
                "EUR",
                Some(uuid::Uuid::new_v4()),
                vec![element(
                    vec![component(ComponentType::ParkingTime, 6.0)],
                    None,
                )],
            );

            normalize_durations(&mut p);

            assert!((p.elements[0].price_components[0].price_excl_vat - 0.1).abs() < 1e-9);
        }

        #[test]
        fn normalize_durations_leaves_parking_time_when_at_or_below_threshold() {
            let mut p = price(
                "EUR",
                Some(uuid::Uuid::new_v4()),
                vec![element(
                    vec![component(ComponentType::ParkingTime, 0.95)],
                    None,
                )],
            );

            normalize_durations(&mut p);

            assert_eq!(p.elements[0].price_components[0].price_excl_vat, 0.95);
        }

        #[test]
        fn normalize_durations_divides_min_duration_when_above_900() {
            let restrictions = Restrictions {
                min_duration: Some(3600),
                max_duration: None,
                start_date: None,
                end_date: None,
                start_time: None,
                end_time: None,
                day_of_week: None,
                min_power: None,
                max_power: None,
            };

            let mut p = price(
                "EUR",
                Some(uuid::Uuid::new_v4()),
                vec![element(
                    vec![component(ComponentType::Energy, 0.5)],
                    Some(restrictions),
                )],
            );

            normalize_durations(&mut p);

            assert_eq!(
                p.elements[0].restrictions.as_ref().unwrap().min_duration,
                Some(60)
            );
        }

        #[test]
        fn normalize_durations_leaves_min_duration_when_at_900() {
            let restrictions = Restrictions {
                min_duration: Some(900),
                max_duration: None,
                start_date: None,
                end_date: None,
                start_time: None,
                end_time: None,
                day_of_week: None,
                min_power: None,
                max_power: None,
            };

            let mut p = price(
                "EUR",
                Some(uuid::Uuid::new_v4()),
                vec![element(
                    vec![component(ComponentType::Energy, 0.5)],
                    Some(restrictions),
                )],
            );

            normalize_durations(&mut p);

            assert_eq!(
                p.elements[0].restrictions.as_ref().unwrap().min_duration,
                Some(900)
            );
        }
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
            if let Some(desc) = &self.description
                && CUSTOMER_ONLY_TARIFFS_NAME.is_match(desc)
            {
                return true;
            }

            CUSTOMER_ONLY_TARIFFS_NAME.is_match(&self.name)
                || CUSTOMER_ONLY_TARIFFS_NAME.is_match(&self.provider_name)
        }
    }

    pub async fn save(
        connection: &mut PgConnection,
        tariff: &Tariff,
        provider_name: &str,
        product_id: uuid::Uuid,
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
            product_id,
        )
        .fetch_one(&mut *connection)
        .await
    }

    pub async fn get_all(connection: &mut PgConnection) -> Result<Vec<EcoTariff>, sqlx::Error> {
        sqlx::query_file_as!(EcoTariff, "sql/get/eco_movement/all_tariff.sql")
            .fetch_all(&mut *connection)
            .await
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn eco_tariff(
            name: &str,
            description: Option<&str>,
            provider_name: &str,
            tariff_type: TariffType,
            subscription_fee: Option<f64>,
        ) -> EcoTariff {
            EcoTariff {
                network: uuid::Uuid::new_v4(),
                name: name.to_string(),
                description: description.map(str::to_string),
                tariff_type,
                provider_name: provider_name.to_string(),
                subscription_fee,
            }
        }

        #[test]
        fn is_ad_hoc_true_when_type_adhoc() {
            let t = eco_tariff("X", None, "Y", TariffType::Adhoc, Some(0.0));
            assert!(t.is_ad_hoc());
        }

        #[test]
        fn is_ad_hoc_false_for_msp() {
            let t = eco_tariff("X", None, "Y", TariffType::Msp, Some(0.0));
            assert!(!t.is_ad_hoc());
        }

        #[test]
        fn is_standard_false_when_subscription_fee_positive() {
            let t = eco_tariff("Neutral", None, "Neutral", TariffType::Msp, Some(5.0));
            assert!(!t.is_standard());
        }

        #[test]
        fn is_standard_true_when_fee_zero_and_not_customer_only() {
            let t = eco_tariff("Neutral", None, "Neutral", TariffType::Msp, Some(0.0));
            assert!(t.is_standard());
        }

        #[test]
        fn is_standard_false_when_customer_only_match() {
            let t = eco_tariff("BMW Business", None, "Neutral", TariffType::Msp, Some(0.0));
            assert!(!t.is_standard());
        }

        #[test]
        fn is_customer_only_matches_description() {
            let t = eco_tariff(
                "Neutral",
                Some("Nur für Kunden"),
                "Neutral",
                TariffType::Msp,
                Some(0.0),
            );

            assert!(t.is_customer_only());
        }

        #[test]
        fn is_customer_only_matches_provider_name() {
            let t = eco_tariff("Neutral", None, "Audi e-tron", TariffType::Msp, Some(0.0));
            assert!(t.is_customer_only());
        }

        #[test]
        fn is_customer_only_false_when_no_match() {
            let t = eco_tariff("Neutral", None, "Neutral", TariffType::Msp, Some(0.0));
            assert!(!t.is_customer_only());
        }
    }
}

pub mod dynamic_price {
    use crate::ladefuchs_db::{
        dynamic_price::{EcoDynamicPrice, EcoLocation},
        plug::ChargeType,
    };
    use sqlx::PgConnection;

    pub async fn get_locations(
        connection: &mut PgConnection,
    ) -> Result<Vec<EcoLocation>, sqlx::Error> {
        let rows = sqlx::query_file_as!(EcoLocation, "sql/get/eco_movement/get_locations.sql")
            .fetch_all(&mut *connection)
            .await?;

        Ok(rows)
    }

    pub async fn get_dynamic_prices(
        connection: &mut PgConnection,
    ) -> Result<Vec<EcoDynamicPrice>, sqlx::Error> {
        let rows = sqlx::query_file_as!(
            EcoDynamicPrice,
            "sql/get/eco_movement/get_dynamic_prices.sql"
        )
        .fetch_all(&mut *connection)
        .await?;

        Ok(rows)
    }
}

pub async fn truncate(connection: &mut PgConnection, table: Table) -> Result<(), sqlx::Error> {
    let query = format!("TRUNCATE TABLE eco_movement.{} cascade", table);
    sqlx::query(&query).execute(connection).await?;
    Ok(())
}
