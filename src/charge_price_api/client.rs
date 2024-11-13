use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use eyre::OptionExt;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE};
use serde_json::Value;

use super::{
    request::{
        charge_station::{ChargePoint, ChargeStationStatistic},
        feedback::FeedBackRequest,
        DataWrapper,
    },
    response::{
        advertisement::AdvertisementsResponse,
        charge_station::{ChargeStationResponse, ChargingStationsStatists},
        company::{self, CompanyResponse},
        condition::TariffPriceResponse,
        tariff::{
            Dimension, EmpCompanyAttributes, Provider, TariffDetailsResponses, TariffWithProvider,
        },
    },
};
use crate::{
    charge_price_api::{
        request::tariff::TariffDetailsRequest,
        response::tariff::{IncludedAttributes, TariffDetailsSegments},
    },
    db::{
        charge_price::ChargePrice,
        operator::{self},
        plug::{ChargeType, Plug},
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
    ) -> Result<TariffPriceResponse, eyre::Error> {
        let response_json = self
            .client
            .post(self.build_url("v1/tariff_details"))
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?
            .json::<TariffDetailsResponses>()
            .await?;

        if response_json.data.is_empty() {
            return Err(eyre::Error::msg(format!(
                "could not fetch tariffs/prices for operator: {}",
                request_body.data.operator.name
            )));
        }

        let charge_type = request_body
            .data
            .attributes
            .station
            .charge_point
            .plug
            .into();

        let mut charge_prices: Vec<ChargePrice> = vec![];

        let operator = Arc::new(request_body.data.operator);

        for response in response_json
            .data
            .iter()
            .filter(|resp| !resp.attributes.restricted_segments.is_empty())
        {
            let mut charge_price = ChargePrice {
                operator_network: operator.clone().network,
                tariff_relation: response.relationships.tariff.data.id,
                c_type: charge_type,
                price: 0.0,
                blocking_fee_start: 0,
                blocking_fee: 0.0,
            };

            let segments = &response.attributes.restricted_segments;

            if segments.iter().any(|s| s.dimension == Dimension::Session) {
                tracing::debug!(
                    reason = "found session dimension skip",
                    operator_name = &operator.name
                );
                continue;
            }

            for TariffDetailsSegments {
                dimension,
                price,
                range_gte,
                ..
            } in segments.iter()
            {
                let price = price.clone();
                match dimension {
                    Dimension::Kwh => {
                        charge_price.price = price;
                    }
                    Dimension::Minute => {
                        charge_price.blocking_fee_start = range_gte.unwrap_or_default();
                        charge_price.blocking_fee = price;
                    }
                    _ => {}
                }
            }

            if charge_price.price == 0.0 {
                continue;
            }
            charge_prices.push(charge_price);
        }

        let providers: HashMap<uuid::Uuid, EmpCompanyAttributes> = response_json
            .included
            .iter()
            .filter_map(|item| match &item.attributes {
                IncludedAttributes::Company(emp) => Some((item.id, emp.clone())),
                _ => None,
            })
            .collect();

        let tariffs: Vec<TariffWithProvider> = response_json
            .included
            .iter()
            .filter_map(|item| {
                if let (IncludedAttributes::Tariff(tariff), Some(emp_relation)) =
                    (&item.attributes, &item.relationships)
                {
                    let provider_id = emp_relation.emp.data.id;
                    let provider = providers.get(&provider_id).map(|emp_attr| Provider {
                        id: provider_id,
                        name: emp_attr.name.clone(),
                    })?;

                    return Some(TariffWithProvider {
                        operator: operator.clone(),
                        id: item.id,
                        attributes: tariff.clone(),
                        provider,
                    });
                }
                None
            })
            .collect();

        Ok(TariffPriceResponse {
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

    pub async fn fetch_all_tariff_prices(
        &self,
        operators: &[operator::admin::Operator],
    ) -> TariffPriceResponse {
        let mut tariff_price_wrapper = TariffPriceResponse {
            charge_prices: Vec::with_capacity(operators.len()),
            tariffs: Vec::with_capacity(operators.len()),
        };

        let mut seen_tariff_ids: HashSet<uuid::Uuid> = HashSet::with_capacity(operators.len());

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
                    data: TariffDetailsRequest::new(operator.clone(), charge_point.clone()),
                };
                match self.fetch_tariff_detail(request).await {
                    Ok(response) => {
                        let tariffs_with_prices = response
                            .charge_prices
                            .iter()
                            .map(|cp| cp.tariff_relation)
                            .collect::<HashSet<_>>();
                        for tariff in response.tariffs {
                            if seen_tariff_ids.insert(tariff.id)
                                && tariffs_with_prices.contains(&tariff.id)
                            {
                                tariff_price_wrapper.tariffs.push(tariff);
                            }
                        }

                        tariff_price_wrapper
                            .charge_prices
                            .extend(response.charge_prices);
                    }
                    Err(err) => {
                        tracing::debug!(
                            context = "fetch_all_tariff_prices",
                            %err,
                            operator_name = operator.name,
                            operator_network = %operator.network,
                            charge_point = %charge_point
                        );
                    }
                }
            }
        }
        tariff_price_wrapper
    }
}
