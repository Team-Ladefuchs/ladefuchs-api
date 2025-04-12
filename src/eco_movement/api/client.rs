use ::serde::de::DeserializeOwned;
use axum::http::{HeaderMap, HeaderValue};
use reqwest::header::AUTHORIZATION;

use super::{location::LocationResponse, price::ConnectorPrice};

pub const LIMIT_OFFSET_PAGE: usize = 1_000;
pub const MAX_PER_PAGE: usize = LIMIT_OFFSET_PAGE * 1_000;

#[derive(Clone, Debug)]
pub struct EcoMovementClient {
    client: reqwest::Client,
    api_url: url::Url,
}

#[derive(Debug, strum_macros::Display)]

enum Endpoint {
    #[strum(to_string = "location")]
    Location,
    #[strum(to_string = "connector_prices")]
    ConnectorPrices,
}

impl EcoMovementClient {
    pub fn new(api_url: url::Url, api_token: &str) -> Self {
        let mut headers = HeaderMap::new();

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_token)).expect("Invalid token format"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();

        Self { client, api_url }
    }

    fn build_url(&self, path: &str) -> url::Url {
        let mut endpoint = self.api_url.clone();
        endpoint.set_path(path);
        endpoint
    }

    pub async fn fetch_location_page(
        &self,
        offset: usize,
    ) -> Result<LocationResponse, reqwest::Error> {
        self.fetch_page(Endpoint::Location, offset).await
    }

    pub async fn fetch_connector_prices(
        &self,
        offset: usize,
    ) -> Result<ConnectorPrice, reqwest::Error> {
        self.fetch_page(Endpoint::ConnectorPrices, offset).await
    }

    async fn fetch_page<T>(&self, endpoint: Endpoint, offset: usize) -> Result<T, reqwest::Error>
    where
        T: DeserializeOwned,
    {
        self.client
            .get(self.build_url(&format!("api/ocpi/cpo/2.1.1/{}", endpoint.to_string())))
            .query(&[("limit", LIMIT_OFFSET_PAGE), ("offset", offset)])
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await
    }
}
