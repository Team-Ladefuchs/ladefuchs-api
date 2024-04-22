use std::collections::HashMap;

use eyre::OptionExt;
use futures_util::{
    future::{self},
    stream::TryStreamExt,
    StreamExt,
};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE};
use serde_json::Value;

use super::{
    request::{DataWrapper, PriceRequest, TariffDetailsRequest},
    response::{AdvertisementsResponse, ChargeStationResponse, CompanyResponse, PricesResponse},
};
use crate::{
    charge_price_api::{
        request::PriceRelationship,
        response::{ApiResponse, ChargeStation, CompanyResult, Dimension, TariffDetailsResponses},
    },
    db::{
        charge_price::ChargePrice,
        operator::{self},
        plug::{ChargeType, Plug},
        tariff::{PriceTuple, TariffBlockingPrice},
        vehicle::Vehicle,
    },
};

const MAX_CONCURRENT_CONNECTIONS: usize = 32;

#[derive(Clone, Debug)]
pub struct ChargePriceAPI {
    client: reqwest::Client,
    api_url: url::Url,
}

pub type ChargingStationsStatists = HashMap<uuid::Uuid, ChargeStation>;

const COUNTRY_FILTER: (&str, &str) = ("filter[country]", "DE");

impl ChargePriceAPI {
    pub fn new(api_url: url::Url, api_token: &str) -> Self {
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

    fn build_url(&self, path: &str) -> url::Url {
        let mut endpoint = self.api_url.clone();
        endpoint.set_path(path);
        endpoint
    }

    pub async fn fetch_all_prices(
        &self,
        operators: &[operator::admin::Operator],
        vehicles: &[Vehicle],
    ) -> Result<Vec<ApiResponse>, eyre::Error> {
        let tasks = operators
            .iter()
            .into_iter()
            .flat_map(|cpo| Self::price_request_payload(&cpo, &vehicles))
            .map(|request| {
                let client = self.clone();
                tokio::task::spawn(async move { client.fetch_price(&request).await })
            });

        let responses = future::try_join_all(tasks)
            .await?
            .into_iter()
            .filter_map(|item| item.ok())
            .collect::<Vec<_>>();
        Ok(responses)
    }

    async fn fetch_price(
        &self,
        body: &DataWrapper<PriceRequest>,
    ) -> Result<ApiResponse, eyre::Error> {
        let data = &body.data;

        let response = self
            .client
            .post(self.build_url("v1/charge_prices"))
            .json(&body)
            .send()
            .await?
            .error_for_status();

        match response {
            Ok(response_value) => {
                let json = response_value.json::<PricesResponse>().await?;
                Ok(ApiResponse {
                    operator: data.operator.clone(),
                    providers: json
                        .data
                        .into_iter()
                        .filter(|msp| {
                            msp.attributes
                                .charge_point_prices
                                .iter()
                                .all(|charge_price| {
                                    charge_price.price_distribution.kwh == Some(1.0)
                                })
                        })
                        .collect::<_>(),
                })
            }
            Err(error) => {
                let err_msg = format!(
                    "could not get prices for CPO: {}\nreason: {}",
                    data.operator.slug_name, error
                );
                Err(eyre::Error::msg(err_msg))
            }
        }
    }

    fn price_request_payload(
        cpo: &operator::admin::Operator,
        vehicles: &[Vehicle],
    ) -> Vec<DataWrapper<PriceRequest>> {
        let mut requests = vehicles
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

    pub async fn fetch_advertisements(&self) -> Result<AdvertisementsResponse, eyre::Error> {
        let response = self
            .client
            .get(self.build_url("v1/advertisements"))
            .query(&[COUNTRY_FILTER, ("exclusive_ad_provider", "true")])
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        let json = response
            .pointer("/data/0/attributes")
            .ok_or_eyre("could not fetch charge price advertisements")?
            .clone();

        serde_json::from_value::<AdvertisementsResponse>(json).map_err(|err| eyre::Error::new(err))
    }

    pub async fn fetch_operator_charging_stations(
        &self,
    ) -> Result<ChargingStationsStatists, eyre::Error> {
        let response = self
            .client
            .get(self.build_url("v1/charging_stations/statistics"))
            .query(&[COUNTRY_FILTER])
            .send()
            .await?
            .error_for_status()?
            .json::<ChargeStationResponse>()
            .await?;

        let mut station_statistics = HashMap::with_capacity(response.data.len());

        let stations = response.data.into_iter().filter(|item| {
            let plug = &item.attributes.plug;
            plug == "ccs" || plug == "type2"
        });

        for station_data in stations {
            if let Some(operator_id) = station_data.relationships.pointer("/operator/data/id") {
                if let Ok(id) = uuid::Uuid::try_parse(operator_id.as_str().unwrap_or_default()) {
                    station_statistics
                        .entry(id)
                        .and_modify(|old: &mut ChargeStation| {
                            match station_data.attributes.plug.parse() {
                                Ok(Plug::CCS) => old.ccs_count += station_data.attributes.count,
                                Ok(Plug::TYPE2) => old.type2_count += station_data.attributes.count,
                                Err(()) => {}
                            }
                        })
                        .or_insert_with(|| ChargeStation {
                            operator_id: id,
                            ccs_count: 0,
                            type2_count: 0,
                        });
                }
            }
        }
        Ok(station_statistics)
    }

    pub async fn fetch_operator(&self) -> Result<Vec<CompanyResult>, eyre::Error> {
        let mut results = vec![];
        let mut page: u8 = 1;
        loop {
            let response = self
                .client
                .get(self.build_url("v1/companies"))
                .query(&[("page[number]", page), ("page[size]", 100)])
                .send()
                .await?
                .error_for_status()?
                .json::<CompanyResponse>()
                .await?;
            let companies = response.data;
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
        body: DataWrapper<TariffDetailsRequest>,
    ) -> Result<Vec<TariffBlockingPrice>, eyre::Error> {
        let json = self
            .client
            .post(self.build_url("v1/tariff_details"))
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<TariffDetailsResponses>()
            .await?;

        if json.data.is_empty() {
            return Ok(vec![]);
        }

        let mut tariff_details = vec![];
        for response in json
            .data
            .iter()
            .filter(|resp| !resp.attributes.restricted_segments.is_empty())
        {
            let dimensions = response
                .attributes
                .restricted_segments
                .iter()
                .filter(|item| item.dimension == Dimension::Minute)
                .take(2)
                .collect::<Vec<_>>();

            let ac_dc_prices = dimensions
                .iter()
                .all(|item| item.charge_point_energy_type.is_none());

            if ac_dc_prices {
                let price = dimensions.first().map(|d| d.price).unwrap_or_default();
                let blocking_ac_tariff = TariffBlockingPrice {
                    operator_network: body.data.operator_network,
                    blocking_fee: price,
                    plug: ChargeType::AC,
                    tariff_relation: response.relationships.tariff.data.id,
                };

                tariff_details.push(blocking_ac_tariff.clone());
                tariff_details.push(TariffBlockingPrice {
                    plug: ChargeType::DC,
                    ..blocking_ac_tariff
                });
            }
            dimensions
                .iter()
                .filter_map(|item| {
                    item.charge_point_energy_type
                        .map(|plug| TariffBlockingPrice {
                            blocking_fee: item.price,
                            plug,
                            tariff_relation: response.relationships.tariff.data.id,
                            operator_network: body.data.operator_network,
                        })
                })
                .for_each(|item| tariff_details.push(item));
        }

        Ok(tariff_details)
    }

    pub async fn fetch_all_tariff_details(
        &self,
        prices: HashMap<uuid::Uuid, Vec<ChargePrice>>,
    ) -> Result<HashMap<PriceTuple, f64>, eyre::Error> {
        tracing::info!(status = "Start fetching tariff details");

        let requests = prices
            .into_iter()
            .map(|(key, value)| DataWrapper {
                data: TariffDetailsRequest::new(key, value),
            })
            .map(|request| self.fetch_tariff_detail(request));

        let tariff_details = futures_util::stream::iter(requests)
            .buffer_unordered(MAX_CONCURRENT_CONNECTIONS)
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .flatten()
            .map(|item| {
                (
                    PriceTuple(item.operator_network, item.tariff_relation, item.plug),
                    item.blocking_fee,
                )
            })
            .collect::<_>();

        tracing::info!(status = "finish tariff details");

        Ok(tariff_details)
    }
}
