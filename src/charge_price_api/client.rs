use super::{request::RequestPayload, response::MSPApiResult};
use crate::{
    charge_price_api::{
        request::Relationship,
        response::{ApiResponse, ResponseError},
    },
    config::Config,
    db::{
        cpo::{self, CPO},
        vehicle::Vehicle,
    },
};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc},
};

pub struct ChargePriceAPI {
    client: reqwest::Client,
    api_url: url::Url,
}

pub struct ApiResult {
    pub charge_point_prices: u64,
    pub responses: Vec<ApiResponse>,
}

impl ChargePriceAPI {
    pub fn new(config: &Config) -> Result<Self, eyre::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_LANGUAGE, "de".parse().unwrap());
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
        );
        headers.insert("API-Key", config.charge_price_api_key.parse()?);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Self {
            client,
            api_url: url::Url::from(config.charge_price_api_url.clone()),
        })
    }

    pub async fn fetch_data(
        client: Arc<ChargePriceAPI>,
        cpos: &[CPO],
        vehicles: &[Vehicle],
    ) -> Result<ApiResult, eyre::Error> {
        let tasks = cpos
            .iter()
            .into_iter()
            .flat_map(|cpo| Self::request_payload(&cpo, &vehicles))
            .map(|request| {
                let client = client.clone();
                tokio::task::spawn(async move { client.do_api_call(&request).await })
            })
            .collect::<Vec<_>>();

        let prices_size = AtomicU64::new(0);
        let responses = futures_util::future::try_join_all(tasks)
            .await?
            .into_iter()
            .flat_map(|result| match result {
                Ok(api_result) => {
                    let prices = api_result
                        .msps
                        .iter()
                        .map(|msp| msp.attributes.charge_point_prices.len() as u64)
                        .sum();
                    prices_size.fetch_add(prices, std::sync::atomic::Ordering::SeqCst);
                    Some(api_result)
                }
                Err(error) => {
                    tracing::error!(error=%error);
                    None
                }
            })
            .collect();

        Ok(ApiResult {
            charge_point_prices: prices_size.into_inner(),
            responses,
        })
    }

    async fn do_api_call(&self, payload: &RequestPayload) -> Result<ApiResponse, eyre::Error> {
        let mut body = HashMap::new();
        body.insert("data", payload.clone());
        let ret = self
            .client
            .post(self.api_url.clone())
            .json(&body)
            .send()
            .await?;

        let status_code = ret.status();
        if status_code.ne(&reqwest::StatusCode::OK) {
            let json: HashMap<String, Value> = ret.json().await?;
            return match json.get("errors") {
                Some(err) => {
                    let errors: Vec<ResponseError> = serde_json::from_value(err.to_owned())?;

                    let err_msg = format!(
                        "could not get prices for CPO: {} status: {} errors: {:#?}",
                        payload.cpo_name, status_code, errors
                    );
                    Err(eyre::Error::msg(err_msg))
                }
                None => Err(unknown_response(&payload.cpo_name)),
            };
        }

        let json: HashMap<String, Value> = ret.json().await?;
        match json.get("data") {
            Some(msps_values) => Ok(ApiResponse {
                cpo_id: payload.cpo_id,
                msps: serialize_msps_result(msps_values)?,
            }),

            None => Err(unknown_response(&payload.cpo_name)),
        }
    }

    fn request_payload(cpo: &cpo::CPO, vehicles: &[Vehicle]) -> Vec<RequestPayload> {
        let mut requests = vec![RequestPayload::new(cpo, Relationship::default())];
        let mut requests_with_vehicle = vehicles
            .clone()
            .into_iter()
            .map(|vehicle| {
                let relationships = Relationship::new(vehicle.id, vehicle.tariff_id);
                RequestPayload::new(cpo, relationships)
            })
            .collect();
        tracing::debug!("{:?}", request = &requests);
        requests.append(&mut requests_with_vehicle);
        requests
    }
}

fn unknown_response(cpo_name: &str) -> eyre::Error {
    eyre::Error::msg(format!("Unkown API response for CPO: {}", cpo_name))
}

fn serialize_msps_result(json: &Value) -> Result<Vec<MSPApiResult>, serde_json::Error> {
    let values = serde_json::from_value::<Vec<MSPApiResult>>(json.to_owned())?;

    let msps = values
        .into_iter()
        .filter(|m| !m.attributes.tariff_name.to_lowercase().contains("business"))
        .filter(|m| {
            m.attributes
                .charge_point_prices
                .iter()
                .map(|t| t.price_distribution.kwh)
                .any(|kwh| kwh == Some(1.0) || kwh.is_none())
        })
        .collect();
    Ok(msps)
}
