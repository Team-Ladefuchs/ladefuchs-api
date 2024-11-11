use std::collections::{HashMap, HashSet};

use eyre::OptionExt;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE};
use serde_json::Value;

use super::{
    request::{
        charge_station::{ChargePoint, ChargeStationStatistic},
        condition::PriceRequest,
        feedback::FeedBackRequest,
        DataWrapper,
    },
    response::{
        self,
        advertisement::AdvertisementsResponse,
        charge_station::{ChargeStationResponse, ChargingStationsStatists},
        company::{self, CompanyResponse},
        condition::{ApiPriceResponse, PricesResponse},
        tariff::{Dimension, TariffDetailsResponses, TariffIncluded},
    },
};
use crate::{
    charge_price_api::{
        request::{condition::PriceRelationship, tariff::TariffDetailsRequest},
        response::tariff::{IncludedAttributes, TariffDetailsSegments},
    },
    db::{
        charge_price::ChargePrice,
        operator::{self},
        plug::{ChargeType, Plug},
        tariff::{PriceTuple, TariffBlockingPrice},
        vehicle::Vehicle,
    },
};

#[derive(Clone, Debug)]
pub struct ChargePriceAPI {
    client: reqwest::Client,
    api_url: url::Url,
}

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
            .timeout(std::time::Duration::from_secs(12))
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
    ) -> Result<Vec<ApiPriceResponse>, eyre::Error> {
        let mut requests = Vec::with_capacity(operators.len());
        for operator in operators {
            requests.extend(vehicles.into_iter().map(|vehicle| {
                let relationship = PriceRelationship::new(vehicle.id, vehicle.tariff_id);
                DataWrapper {
                    data: PriceRequest::new(operator, relationship),
                }
            }));

            requests.push(DataWrapper {
                data: PriceRequest::new(operator, PriceRelationship::default()),
            });
        }

        tracing::debug!(?requests);
        // requests
        let mut responses = Vec::with_capacity(operators.len());
        for request in requests {
            match self.fetch_price(request).await {
                Ok(response) => responses.push(response),
                Err(err) => tracing::error!(context="fetch all prices", %err),
            }
        }

        Ok(responses)
    }

    async fn fetch_price(
        &self,
        body: DataWrapper<PriceRequest>,
    ) -> Result<ApiPriceResponse, eyre::Error> {
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
                Ok(ApiPriceResponse {
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
                    "could not get prices for CPO: {}\n: request_body: {}",
                    data.operator.slug_name,
                    serde_json::to_string_pretty(&body).unwrap_or_default()
                );
                Err(eyre::Error::from(error).wrap_err(err_msg))
            }
        }
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

        let mut station_statistics: HashMap<uuid::Uuid, ChargeStationStatistic> =
            HashMap::with_capacity(response.data.len());

        let stations = response.data.into_iter().filter(|item| {
            let plug = &item.attributes.plug;
            plug == "ccs" || plug == "type2"
        });

        for station_data in stations {
            let operator_id = match station_data.relationships.pointer("/operator/data/id") {
                Some(id) => id.as_str().unwrap_or_default(),
                None => continue,
            };

            let id = match uuid::Uuid::try_parse(operator_id) {
                Ok(uuid) => uuid,
                Err(_) => continue,
            };

            let plug = match station_data.attributes.plug.parse() {
                Ok(plug) => plug,
                Err(_) => continue,
            };

            station_statistics
                .entry(id)
                .and_modify(|old: &mut ChargeStationStatistic| match plug {
                    Plug::CCS => old.ccs_count += station_data.attributes.count,
                    Plug::TYPE2 => old.type2_count += station_data.attributes.count,
                })
                .or_insert_with(|| {
                    let mut statistic = ChargeStationStatistic {
                        ccs_count: 0,
                        type2_count: 0,
                    };
                    match plug {
                        Plug::CCS => statistic.ccs_count = station_data.attributes.count,
                        Plug::TYPE2 => statistic.type2_count = station_data.attributes.count,
                    }
                    statistic
                });
        }

        Ok(station_statistics)
    }

    pub async fn fetch_operator(&self) -> Result<Vec<company::CompanyResult>, eyre::Error> {
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
        request_body: DataWrapper<TariffDetailsRequest>,
    ) -> Result<TariffPriceWrapper, eyre::Error> {
        let response_json = self
            .client
            .post(self.build_url("v1/tariff_details"))
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?
            .json::<TariffDetailsResponses>()
            .await?;

        // todo
        // if response_json.data.is_empty() {
        //     return Ok(vec![]);
        // }

        let charge_type = request_body
            .data
            .attributes
            .station
            .charge_point
            .plug
            .into();

        let mut charge_prices = vec![];

        for response in response_json
            .data
            .iter()
            .filter(|resp| !resp.attributes.restricted_segments.is_empty())
        {
            let mut charge_price = ChargePrice {
                operator_id: 0,
                operator_network: request_body.data.attributes.station.operator.id,
                tariff_relation: response.relationships.tariff.data.id,
                tariff_id: 0,
                c_type: charge_type,
                price: 0.0,
                blocking_fee_start: 0,
                blocking_fee: 0.0,
            };
            for TariffDetailsSegments {
                dimension,
                price,
                range_gte,
                ..
            } in response.attributes.restricted_segments.clone()
            {
                match dimension {
                    Dimension::Kwh => {
                        charge_price.price = price;
                    }
                    Dimension::Minute => {
                        charge_price.blocking_fee_start = range_gte.unwrap_or_default();
                        charge_price.blocking_fee = price;
                    }
                    Dimension::Session => {}
                }
            }
            charge_prices.push(charge_price);
        }

        let tariffs: Vec<TariffIncluded> = response_json
            .included
            .into_iter()
            .filter_map(|included| {
                if let IncludedAttributes::Tariff(_) = included.attributes {
                    Some(included)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok(TariffPriceWrapper {
            charge_prices,
            tariffs,
        })
    }

    pub async fn send_feedback(&self, body: &FeedBackRequest) -> Result<(), eyre::Error> {
        self.client
            .post(self.build_url("v1/user_feedback"))
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn fetch_all_tariff_prices(&self, operators: &[operator::admin::Operator]) {
        // let mut response = Vec::with_capacity(operators.len());
        for operator in operators {
            for plug in &operator.supported_types {
                let charge_point = match plug {
                    ChargeType::AC => ChargePoint {
                        power: operator.power_ac,
                        plug: Plug::TYPE2,
                    },
                    ChargeType::DC => ChargePoint {
                        power: operator.power_dc,
                        plug: Plug::CCS,
                    },
                };
                let request = DataWrapper {
                    data: TariffDetailsRequest::new(operator.network, charge_point),
                };
                // dbg!(response);
                match self.fetch_tariff_detail(request).await {
                    Ok(value2) => {
                        dbg!(value2);
                    }
                    Err(err) => {
                        dbg!(err);
                    }
                }
                // response.extend(self.fetch_tariff_detail(request).await);
            }
        }
    }

    pub async fn fetch_all_tariff_details(
        &self,
        prices: &[ChargePrice],
    ) -> Result<HashMap<PriceTuple, f64>, eyre::Error> {
        tracing::info!(
            status = "Start fetching tariff details for prices",
            count = prices.len()
        );

        // let requests = prices
        //     .iter()
        //     .filter(|price| price.blocking_fee_start > 0)
        //     .fold(HashMap::new(), |mut map, price| {
        //         map.entry(price.operator_network)
        //             .or_insert_with(HashSet::new)
        //             .insert(price.tariff_relation);
        //         map
        //     })
        //     .into_iter()
        //     .map(|(operator_id, tariffs_ids)| DataWrapper {
        //         data: TariffDetailsRequest::new(operator_id, Ha),
        //     })
        //     .collect::<Vec<_>>(); //
        // let requests = vec![];
        // let mut responses = Vec::with_capacity(requests.len());
        // for request in requests {
        //     match self.fetch_tariff_detail(request).await {
        //         Ok(response) => responses.extend(response),
        //         Err(error) => tracing::error!(context="fetch all tariff details", %error),
        //     }
        // }

        // let tariff_details = responses
        //     .into_iter()
        //     .map(|item| {
        //         (
        //             PriceTuple(item.operator_network, item.tariff_relation, item.plug),
        //             item.blocking_fee,
        //         )
        //     })
        //     .collect::<HashMap<PriceTuple, f64>>();

        // tracing::info!(
        //     status = "Finish tariff details",
        //     count = tariff_details.len()
        // );

        Ok(HashMap::new())
    }
}

// todo
#[derive(Debug, Clone)]
pub struct TariffPriceWrapper {
    pub charge_prices: Vec<ChargePrice>,
    pub tariffs: Vec<TariffIncluded>,
}
