use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConnectorPriceResponse {
    #[serde(default)]
    pub data: Vec<ConnectorPrice>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConnectorPrice {
    pub location_id: uuid::Uuid,
    pub evse_uid: String,
    pub connector_id: String,
    #[serde(default)]
    pub pricing_ids: Vec<uuid::Uuid>,
}
