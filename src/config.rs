use std::net::IpAddr;

use serde::Deserialize;

use crate::log::LogType;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(rename(serialize = "DATABASE_URL"))]
    pub database_url: url::Url,
    #[serde(rename(serialize = "CHARGE_PRICE_API_KEY"))]
    pub charge_price_api_key: String,
    #[serde(rename(serialize = "CHARGE_PRICE_API_URL"))]
    pub charge_price_api_url: url::Url,
    #[serde(default)]
    #[serde(rename(serialize = "LOG_TYPE"))]
    pub log_type: LogType,
    #[serde(default = "default_port")]
    #[serde(rename(serialize = "PORT"))]
    pub port: u16,
    #[serde(rename(serialize = "ADDRESS"))]
    #[serde(default = "default_address")]
    pub address: IpAddr,
    #[serde(rename(serialize = "INTERVAL_V"))]
    #[serde(default = "default_interval_h")]
    pub interval_h: u8,
    #[serde(rename(serialize = "AUTH_TOKEN"))]
    pub auth_token: String,
}

fn default_port() -> u16 {
    3000
}

fn default_interval_h() -> u8 {
    6
}

fn default_address() -> IpAddr {
    [127, 0, 0, 1].into()
}

pub fn read_config() -> Result<Config, envy::Error> {
    envy::from_env()
}
