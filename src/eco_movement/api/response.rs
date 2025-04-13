use serde::Deserialize;
use serde::Serialize;

use crate::eco_movement::api::client::ResponseData;

pub mod location {

    use super::*;

    pub type LocationResponse = ResponseData<LocationData>;

    #[derive(Debug, Deserialize, Serialize, Default)]
    pub struct LocationData {
        pub id: uuid::Uuid,
        #[serde(flatten)]
        pub value: serde_json::Value,
    }
}

pub mod price {
    use super::*;
    pub type ConnectorPriceResponse = ResponseData<ConnectorPrice>;

    #[derive(Debug, Deserialize, Serialize, Default)]
    pub struct ConnectorPrice {
        pub location_id: uuid::Uuid,
        pub evse_uid: String,
        pub connector_id: String,
        #[serde(default)]
        pub pricing_ids: Vec<String>,
    }

    pub type PriceResponse = ResponseData<PriceData>;

    #[derive(Debug, Deserialize, Serialize, Default)]
    pub struct PriceData {
        pub id: String,
        #[serde(flatten)]
        pub value: serde_json::Value,
    }
}
