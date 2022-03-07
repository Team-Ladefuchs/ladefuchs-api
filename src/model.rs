use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ChargePrice {
    pub identifier: String,
    #[serde(rename = "type")]
    pub provider: String,
    pub name: String,
    pub price: f64,
    pub monthly_fee: f64,
}
