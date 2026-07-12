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
