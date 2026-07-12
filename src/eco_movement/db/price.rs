use super::*;
use crate::{
    eco_movement::api::response::price::{ComponentType, PriceData},
    ladefuchs_db::plug::ChargeType,
};

struct ImportablePrice {
    data: PriceData,
    product_id: uuid::Uuid,
}

pub async fn save_multiple(
    connection: &mut PgConnection,
    prices: Vec<PriceData>,
) -> Result<(), sqlx::Error> {
    let mut importable: Vec<ImportablePrice> = prices
        .into_iter()
        .filter_map(|item| {
            is_importable(&item).map(|product_id| ImportablePrice {
                data: item,
                product_id,
            })
        })
        .collect();

    drop_parking_only_singleton(&mut importable);

    for entry in &mut importable {
        let tariff_id = tariff::save(
            connection,
            &entry.data.tariff,
            &entry.data.provider_name,
            entry.product_id,
        )
        .await?;

        normalize_durations(&mut entry.data);

        save(connection, &entry.data, &tariff_id).await?;
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

fn drop_parking_only_singleton(filtered: &mut Vec<ImportablePrice>) {
    if filtered.len() == 1
        && filtered
            .first()
            .and_then(|entry| entry.data.elements.first())
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

        let mut v = vec![ImportablePrice {
            data: p,
            product_id: id,
        }];
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

        let mut v = vec![
            ImportablePrice {
                data: p1,
                product_id: id,
            },
            ImportablePrice {
                data: p2,
                product_id: id,
            },
        ];
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

        let mut v = vec![ImportablePrice {
            data: p,
            product_id: id,
        }];
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
