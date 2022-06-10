use std::{net::IpAddr, path::PathBuf};

use crate::log::LogType;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(rename(serialize = "DATABASE_URL"))]
    pub database_url: url::Url,
    #[serde(rename(serialize = "DATABASE_POOL_SIZE"))]
    #[serde(default = "default_database_pool_size")]
    pub database_pool_size: u32,
    #[serde(rename(serialize = "CHARGE_PRICE_API_KEY"))]
    pub charge_price_api_key: String,
    #[serde(rename(serialize = "CHARGE_PRICE_API_URL"))]
    #[serde(default = "default_charge_price_api_url")]
    pub charge_price_api_url: url::Url,
    #[serde(default)]
    #[serde(rename(serialize = "LOG_TYPE"))]
    pub log_type: LogType,
    #[serde(default = "default_port")]
    #[serde(rename(serialize = "PORT"))]
    pub port: u16,
    #[serde(rename(serialize = "LISTEN"))]
    #[serde(default = "default_listen")]
    pub listen: IpAddr,
    #[serde(rename(serialize = "INTERVAL"))]
    #[serde(default = "default_interval_h")]
    pub interval_h: u8,
    #[serde(rename(serialize = "AUTH_TOKEN"))]
    pub auth_token: String,
    #[serde(rename(serialize = "DOMAIN"))]
    pub domain: url::Url,
    #[serde(default = "default_image_folder")]
    #[serde(rename(serialize = "IMAGE_PATH"))]
    pub image_folder: PathBuf,
}

fn default_charge_price_api_url() -> url::Url {
    "https://api.chargeprice.app/v1/charge_prices"
        .parse()
        .unwrap()
}

fn default_domain() -> url::Url {
    let mut url = url::Url::parse("http://localhost").unwrap();
    url.set_port(Some(default_port())).unwrap();
    url
}

fn default_port() -> u16 {
    3000
}

fn default_interval_h() -> u8 {
    3
}

fn default_database_pool_size() -> u32 {
    8
}

fn default_listen() -> IpAddr {
    [127, 0, 0, 1].into()
}

fn default_image_folder() -> PathBuf {
    PathBuf::from("./cards")
}

pub fn read_config() -> Result<Config, envy::Error> {
    envy::from_env()
}
