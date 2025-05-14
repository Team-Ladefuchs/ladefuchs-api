use std::{net::IpAddr, path::PathBuf};

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(rename(serialize = "DATABASE_URL"))]
    pub database_url: url::Url,
    #[serde(rename(serialize = "DATABASE_POOL_SIZE"))]
    #[serde(default = "default_database_pool_size")]
    pub database_pool_size: u32,
    #[serde(rename(serialize = "ECO_MOVEMENT_KEY"))]
    pub eco_movement_api_key: String,
    #[serde(rename(serialize = "ECO_MOVEMENT_API_URL"))]
    #[serde(default = "default_eco_movement_url")]
    pub eco_movement_api_url: url::Url,
    #[serde(default = "default_port")]
    #[serde(rename(serialize = "PORT"))]
    pub port: u16,
    #[serde(rename(serialize = "LISTEN"))]
    #[serde(default = "default_listen")]
    pub listen: IpAddr,
    #[serde(rename(serialize = "CRON_SCHEDULE"))]
    #[serde(default = "default_cron_schedule")]
    pub cron_schedule: String,
    #[serde(default = "default_api_domain")]
    #[serde(rename(serialize = "DOMAIN"))]
    pub domain: url::Url,
    #[serde(default = "none_str")]
    #[serde(rename(serialize = "SLACK_CHANNEL"))]
    pub slack_channel: Option<String>,
    #[serde(default = "none_str")]
    #[serde(rename(serialize = "SLACK_TOKEN"))]
    pub slack_token: Option<String>,
    #[serde(default = "none_str")]
    #[serde(rename(serialize = "ADMIN_USER"))]
    pub admin_user: Option<String>,
    #[serde(default = "none_str")]
    #[serde(rename(serialize = "ADMIN_PWD"))]
    pub admin_pwd: Option<String>,
    #[serde(default = "default_admin_domain")]
    #[serde(rename(serialize = "ADMIN_DOMAIN"))]
    pub admin_domain: url::Url,
    #[serde(default = "default_docs_dir")]
    #[serde(rename(serialize = "DOCS_DIR"))]
    pub docs_dir: PathBuf,
    #[serde(rename(serialize = "IMPORT_ON_START"))]
    pub import_on_start: bool,
}

fn default_eco_movement_url() -> url::Url {
    "https://api.eco-movement.com".parse().unwrap()
}

fn default_admin_domain() -> url::Url {
    "http://127.0.0.1:8080".parse().unwrap()
}

fn default_api_domain() -> url::Url {
    let mut url = url::Url::parse("http://127.0.0.1").unwrap();
    url.set_port(Some(default_port())).unwrap();
    url
}

fn default_docs_dir() -> PathBuf {
    PathBuf::from("./docs")
}

fn default_port() -> u16 {
    3000
}

fn default_cron_schedule() -> String {
    String::from("0 45 23 * * *")
}

fn default_database_pool_size() -> u32 {
    12
}

fn default_listen() -> IpAddr {
    [127, 0, 0, 1].into()
}

fn none_str() -> Option<String> {
    None
}

pub fn read_config() -> Result<Config, envy::Error> {
    envy::from_env()
}
