use super::{
    request::{DataWrapper, PriceRequest, TariffDetailsRequest},
    response::MSPApiResult,
};
use crate::{
    charge_price_api::{
        request::PriceRelationship,
        response::{
            ApiDataResponse, ApiResponse, CompanyResult, DimenSion, ResponseError, TariffDetails,
        },
    },
    db::{
        cpo::{self, CPO},
        tariff::{TariffBlockingPrice, TariffsWithBlockingFee},
        vehicle::Vehicle,
    },
};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE};
use serde_json::Value;
use std::collections::HashMap;

use futures_util::future;

#[derive(Clone, Debug)]
pub struct ChargePriceAPI {
    client: reqwest::Client,
    api_url: String,
}

impl ChargePriceAPI {
    pub fn new(api_url: String, api_token: &str) -> Self {
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

    async fn fetch_price(
        &self,
        body: &DataWrapper<PriceRequest>,
    ) -> Result<ApiResponse, eyre::Error> {
        let data = &body.data;

        let ret = self
            .client
            .post(format!("{}v1/charge_prices", self.api_url))
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
                        data.cpo_name, status_code, errors
                    );
                    Err(eyre::Error::msg(err_msg))
                }
                None => Err(unknown_response(&data.cpo_name)),
            };
        }

        match ret
            .json::<ApiDataResponse<MSPApiResult>>()
            .await?
            .results
            .get("data")
        {
            Some(msps_values) => Ok(ApiResponse {
                cpo_id: data.cpo_id,
                cpo_name: data.cpo_name.clone(),
                msps: msps_values.clone(),
            }),

            None => Err(unknown_response(&data.cpo_name)),
        }
    }

    fn price_request_payload(
        cpo: &cpo::CPO,
        vehicles: &[Vehicle],
    ) -> Vec<DataWrapper<PriceRequest>> {
        let mut requests = vehicles
            .clone()
            .into_iter()
            .map(|vehicle| {
                let relationships = PriceRelationship::new(vehicle.id, vehicle.tariff_id);
                DataWrapper {
                    data: PriceRequest::new(cpo, relationships),
                }
            })
            .collect::<Vec<_>>();

        requests.push(DataWrapper {
            data: PriceRequest::new(cpo, PriceRelationship::default()),
        });
        tracing::debug!(?requests);
        requests
    }

    pub async fn fetch_companies(&self) -> Result<Vec<CompanyResult>, eyre::Error> {
        let mut results = vec![];
        let mut page: u8 = 1;

        loop {
            let response = self
                .client
                .get(format!("{}v1/companies", self.api_url))
                .query(&[("page[number]", page), ("page[size]", 100)])
                .send()
                .await?
                .json::<ApiDataResponse<CompanyResult>>()
                .await?;
            let companies = match response.results.get("data") {
                Some(it) => it,
                _ => continue,
            };
            if companies.len() == 0 || page > 50 {
                break;
            }
            let mut companies = companies
                .clone()
                .into_iter()
                .filter(|company| company.attributes.is_cpo)
                .filter(|company| {
                    let is_country_de = company.attributes.cpo_countries.iter().any(|i| i == &"DE");

                    let is_external_de = company
                        .attributes
                        .external_source_mapping
                        .evse_operator_ids
                        .as_ref()
                        .map(|evs| evs.iter().any(|i| i.starts_with("DE*")))
                        .unwrap_or_default();
                    return is_country_de || is_external_de;
                })
                .collect::<Vec<_>>();
            results.append(&mut companies);
            page += 1;
        }
        Ok(results)
    }

    pub async fn fetch_tariff_detail(
        &self,
        body: &DataWrapper<TariffDetailsRequest>,
    ) -> Result<Vec<TariffBlockingPrice>, eyre::Error> {
        let json = self
            .client
            .post(format!("{}v1/tariff_details", self.api_url))
            .json(&body)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let ret = match json.pointer("/data/0/attributes/restricted_segments") {
            Some(value) => {
                let details: Vec<TariffDetails> = serde_json::from_value(value.clone())?;

                let TariffsWithBlockingFee {
                    tariff_id, cpo_id, ..
                } = body.data.context;

                details
                    .iter()
                    .filter(|item| item.dimension == DimenSion::Minute)
                    .filter_map(|item| {
                        item.charge_point_energy_type
                            .map(|plug| TariffBlockingPrice {
                                tariff_id: tariff_id,
                                cpo_id: cpo_id,
                                price: item.price,
                                plug,
                            })
                    })
                    .collect::<Vec<_>>()
            }
            None => {
                tracing::error!("Tariff details were empty. Maybe the json schema has changed?");
                vec![]
            }
        };
        Ok(ret)
    }

    pub async fn fetch_all_tariff_details(
        &self,
        blocking_tariffs: Vec<TariffsWithBlockingFee>,
    ) -> Result<Vec<TariffBlockingPrice>, eyre::Error> {
        let requests = blocking_tariffs
            .into_iter()
            .map(|item| DataWrapper {
                data: TariffDetailsRequest::new(item),
            })
            .collect::<Vec<_>>();

        let responses = requests
            .iter()
            .map(|request| self.fetch_tariff_detail(&request));

        let tariff_details = future::try_join_all(responses)
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        Ok(tariff_details)
    }
}

fn unknown_response(cpo_name: &str) -> eyre::Error {
    eyre::Error::msg(format!("Unknown API response for CPO: {}", cpo_name))
}
