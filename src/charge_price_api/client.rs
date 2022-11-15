use super::{request::RequestPayload, response::MSPApiResult};
use crate::{
    charge_price_api::{
        request::Relationship,
        response::{ApiDataResponse, ApiResponse, CompanyResult, ResponseError},
    },
    db::{
        cpo::{self, CPO},
        vehicle::Vehicle,
    },
};
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE},
    Url,
};
use serde_json::Value;
use std::collections::HashMap;

use futures_util::future;

#[derive(Clone, Debug)]
pub struct ChargePriceAPI {
    client: reqwest::Client,
    api_url: url::Url,
}

impl ChargePriceAPI {
    pub fn new(api_url: Url, api_token: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_LANGUAGE, "de".parse().unwrap());
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
        );
        headers.insert("API-Key", api_token.parse().unwrap());

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Self { client, api_url }
    }

    pub async fn fetch_all_prices(
        &self,
        cpos: &[CPO],
        vehicles: &[Vehicle],
    ) -> Result<Vec<ApiResponse>, eyre::Error> {
        let tasks = cpos
            .iter()
            .into_iter()
            .flat_map(|cpo| Self::price_request_payload(&cpo, &vehicles))
            .map(|request| {
                let client = self.clone();
                tokio::task::spawn(async move { client.fetch_price(&request).await })
            });

        let mut responses = vec![];

        for task in future::try_join_all(tasks).await? {
            match task {
                Ok(api_response) => responses.push(api_response),
                Err(err) => {
                    return Err(err);
                }
            }
        }
        Ok(responses)
    }

    async fn fetch_price(&self, payload: &RequestPayload) -> Result<ApiResponse, eyre::Error> {
        let mut body = HashMap::new();
        body.insert("data", payload.clone());
        let mut price_endpoint = self.api_url.clone();
        price_endpoint.set_path("v1/charge_prices");
        let ret = self.client.post(price_endpoint).json(&body).send().await?;

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

        match ret
            .json::<ApiDataResponse<MSPApiResult>>()
            .await?
            .results
            .get("data")
        {
            Some(msps_values) => Ok(ApiResponse {
                cpo_id: payload.cpo_id,
                cpo_name: payload.cpo_name.clone(),
                msps: msps_values.clone(),
            }),

            None => Err(unknown_response(&payload.cpo_name)),
        }
    }

    fn price_request_payload(cpo: &cpo::CPO, vehicles: &[Vehicle]) -> Vec<RequestPayload> {
        let mut requests = vec![RequestPayload::new(cpo, Relationship::default())];
        let mut requests_with_vehicle = vehicles
            .clone()
            .into_iter()
            .map(|vehicle| {
                let relationships = Relationship::new(vehicle.id, vehicle.tariff_id);
                RequestPayload::new(cpo, relationships)
            })
            .collect();
        tracing::debug!(?requests);
        requests.append(&mut requests_with_vehicle);
        requests
    }
    pub async fn fetch_companies(&self) -> Result<Vec<CompanyResult>, eyre::Error> {
        let mut results = vec![];
        let mut page: u8 = 1;
        let mut company_endpoint = self.api_url.clone();
        company_endpoint.set_path("v1/companies");
        loop {
            let response = self
                .client
                .get(company_endpoint.clone())
                .query(&[("page[number]", page), ("page[size]", 100)])
                .send()
                .await?
                .json::<ApiDataResponse<CompanyResult>>()
                .await?;
            if let Some(companies) = response.results.get("data") {
                if companies.len() == 0 || page > 50 {
                    break;
                }
                let mut companies = companies
                    .clone()
                    .into_iter()
                    .filter(|company| company.attributes.is_cpo)
                    .filter(|company| company.attributes.cpo_countries.iter().any(|i| i == &"DE"))
                    .collect::<Vec<_>>();
                results.append(&mut companies);
                page += 1;
            }
        }
        Ok(results)
    }
}

fn unknown_response(cpo_name: &str) -> eyre::Error {
    eyre::Error::msg(format!("Unkown API response for CPO: {}", cpo_name))
}
