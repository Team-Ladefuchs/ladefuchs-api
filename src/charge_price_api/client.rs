use super::{request::RequestPayload, response::AllChargePrices};
use crate::{
    charge_price_api::{
        request::Relationship,
        response::{ApiResultWrapper, ResponseError},
    },
    config::Config,
    db::{
        cpo::{self, CPO},
        vehicle::Vehicle,
    },
};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

pub struct ChargePriceAPI {
    client: reqwest::Client,
    api_url: url::Url,
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
    ) -> Result<AllChargePrices, eyre::Error> {
        let tasks = cpos
            .iter()
            .into_iter()
            .flat_map(|cpo| Self::request_payload(&cpo, &vehicles))
            .map(|request| {
                let client = client.clone();
                tokio::task::spawn(async move { client.do_api_call(&request).await })
            })
            .collect::<Vec<_>>();

        let ret = futures_util::future::try_join_all(tasks)
            .await?
            .into_iter()
            .filter_map(|result| match result {
                Ok(prices) => Some(prices),
                Err(error) => {
                    tracing::error!(error=%error);
                    None
                }
            })
            .collect();

        Ok(ret)
    }

    async fn do_api_call(&self, payload: &RequestPayload) -> Result<ApiResultWrapper, eyre::Error> {
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
            Some(msps_values) => Ok(ApiResultWrapper {
                cpo_id: payload.cpo_id,
                msps: serde_json::from_value(msps_values.to_owned())?,
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
                let relationships = Relationship::new(vehicle.id, vehicle.tarif_id);
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
