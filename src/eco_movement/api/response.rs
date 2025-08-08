use serde::Deserialize;
use serde::Serialize;

pub mod operator {

    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct Operator {
        #[serde(alias = "partner_id")]
        pub id: uuid::Uuid,
        pub name: String,
        pub website: Option<String>,
        pub ema_id: Vec<String>,
    }
}

pub mod location {

    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct LocationData {
        pub id: uuid::Uuid,
        pub evses: Vec<Evse>,
        // #[serde(deserialize_with = "deserialize_country_from_alpha3")]
        pub country: String,
        pub operator: Option<operator::Operator>,
        #[serde(alias = "type")]
        pub location_type: LocationType,
        #[serde(flatten)]
        pub value: serde_json::Value,
        pub restrictions: Option<Vec<RestrictionType>>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum RestrictionType {
        Customers,
        TimeRestricted,
        #[serde(other)]
        Other,
    }

    #[derive(Debug, Deserialize, sqlx::Type, PartialEq)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    #[sqlx(rename_all = "snake_case")]
    #[sqlx(type_name = "eco_movement.LocationType")]
    pub enum LocationType {
        OnStreet,
        ParkingGarage,
        UndergroundGarage,
        ParkingLot,
        #[serde(other)]
        Other,
    }

    #[derive(Debug, Deserialize)]
    pub struct Connector {
        pub id: String,
        pub power_type: PowerType,
        pub max_power: i32,
        #[serde(alias = "standard")]
        pub connector_type: ConnectorType,
    }

    #[derive(Debug, Deserialize, sqlx::Type, PartialEq)]
    #[sqlx(rename_all = "snake_case")]
    #[sqlx(type_name = "eco_movement.ConnectorType")]
    pub enum ConnectorType {
        #[serde(alias = "IEC_62196_T2")]
        Type2,
        #[serde(alias = "IEC_62196_T2_COMBO")]
        CCS,
        #[serde(alias = "CHADEMO")]
        Chademo,
        #[serde(other)]
        Other,
    }

    #[derive(Debug, Deserialize)]
    pub struct Evse {
        pub uid: String,
        pub connectors: Vec<Connector>,
    }

    #[derive(Debug, sqlx::Type, Deserialize)]
    #[sqlx(rename_all = "snake_case")]
    #[sqlx(type_name = "eco_movement.PowerType")]
    pub enum PowerType {
        #[serde(alias = "AC_3_PHASE")]
        Ac3Phase,
        #[serde(alias = "AC_1_PHASE")]
        Ac1Phase,
        #[serde(alias = "DC")]
        Dc,
        #[serde(other)]
        Other,
    }
}

pub mod price {

    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct ConnectorPrice {
        pub location_id: uuid::Uuid,
        pub evse_uid: String,
        pub connector_id: String,
        #[serde(default)]
        pub pricing_ids: Vec<String>,
    }

    #[derive(Debug, Deserialize)]
    pub struct PriceData {
        pub id: String,
        #[serde(alias = "partner")]
        pub provider_name: String,
        #[serde(alias = "product")]
        pub tariff: tariff::Tariff,
        #[serde(default)]
        pub elements: Vec<Elements>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Elements {
        pub price_components: Vec<Components>,
        pub restrictions: Option<Restrictions>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Restrictions {
        pub min_duration: Option<u32>,
        pub max_duration: Option<u32>,
        pub start_date: Option<String>,
        pub end_date: Option<String>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Components {
        pub price_excl_vat: f64,
        pub vat: i32,
        pub step_size: u32,
        #[serde(alias = "type")]
        pub price_type: ComponentType,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum ComponentType {
        Energy,
        Flat,
        #[serde(alias = "TIME")]
        ParkingTime,
        #[serde(other)]
        Other,
    }
}

pub mod tariff {

    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub struct Tariff {
        pub name: String,
        pub description: String,
        pub subscription_type: String,
        pub subscription_fee_excl_vat: String,
        #[serde(alias = "type")]
        pub _type: TariffType,
        pub currency: String,
    }

    #[derive(Debug, Deserialize, sqlx::Type, PartialEq)]
    #[serde(rename_all = "snake_case")]
    #[sqlx(type_name = "eco_movement.TariffType")]
    #[sqlx(rename_all = "snake_case")]
    pub enum TariffType {
        Msp,
        Adhoc,
        CpoSubscription,
        #[serde(other)]
        Other,
    }
}
