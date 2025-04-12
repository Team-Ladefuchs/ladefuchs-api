use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub struct LocationResponse {
    #[serde(default)]
    pub data: Vec<LocationData>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct LocationData {
    pub id: uuid::Uuid,
    #[serde(flatten)]
    pub value: serde_json::Value,
}
