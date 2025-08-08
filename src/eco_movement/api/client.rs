use std::{any, fmt::Debug};

use ::serde::de::DeserializeOwned;
use async_stream::try_stream;
use axum::http::{HeaderMap, HeaderValue};
use futures_util::Stream;
use reqwest::header::AUTHORIZATION;

#[derive(Clone, Debug)]
pub struct EcoMovementClient {
    client: reqwest::Client,
    api_url: url::Url,
}

const PER_PAGE_SIZE: usize = 1_000;

#[derive(Debug, strum_macros::Display)]
pub enum Endpoint {
    #[strum(to_string = "api/ocpi/cpo/2.1.1/locations")]
    Location,
    #[strum(to_string = "prices/connector_prices")]
    ConnectorPrice,
    #[strum(to_string = "prices")]
    Price,
}

#[derive(serde::Deserialize, Debug)]
pub struct ResponseData<T> {
    pub data: Option<Vec<T>>,
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

    pub async fn fetch_page<T>(
        &self,
        endpoint: Endpoint,
        offset: usize,
    ) -> Result<ResponseData<T>, reqwest::Error>
    where
        T: DeserializeOwned,
    {
        self.client
            .get(self.build_url(&endpoint.to_string()))
            .query(&[("limit", PER_PAGE_SIZE), ("offset", offset)])
            .send()
            .await?
            .error_for_status()?
            .json::<ResponseData<T>>()
            .await
    }
}

pub fn stream_all_data<'a, T, F, Fut>(
    fetch_fn: F,
    max_request_pages: usize,
) -> impl Stream<Item = Result<Vec<T>, reqwest::Error>> + 'a
where
    T: DeserializeOwned + 'a,
    F: Fn(usize) -> Fut + Send + Sync + 'a,
    Fut: std::future::Future<Output = Result<ResponseData<T>, reqwest::Error>> + Send + 'a,
{
    let mut offset = 0;
    let max_total_offset = PER_PAGE_SIZE * max_request_pages;
    try_stream! {

        loop {
            let response = fetch_fn(offset).await?;
            if let Some(data) = response.data{
                tracing::debug!(
                    type = any::type_name::<T>(),
                    offset,
                    data = &data.len()
                );

                offset += PER_PAGE_SIZE;


                if offset > max_total_offset || data.len() < PER_PAGE_SIZE {
                    yield data;
                    break;
                }
                yield data;
            } else {
                break;
            }
        }
    }
}
