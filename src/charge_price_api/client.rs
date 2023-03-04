use futures_util::future;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE};

use super::{
    request::{DataWrapper, PriceRequest, TariffDetailsRequest},
    response::{CompanyResponses, PricesResponse},
};
use crate::{
    charge_price_api::{
        request::PriceRelationship,
        response::{ApiResponse, CompanyResult, DimenSion, TariffDetails},
    },
    db::{
        cpo::{self, CPO},
        plug::ChargeType,
        tariff::{TariffBlockingPrice, TariffsWithBlockingFee},
        vehicle::Vehicle,
    },
};

#[derive(Clone, Debug)]
pub struct ChargePriceAPI {
    client: reqwest::Client,
    api_url: url::Url,
}

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
                    cpo_id: data.cpo_id,
                    cpo_name: data.cpo_name.clone(),
                    msps: json.data,
                })
            }
            Err(error) => {
                let err_msg = format!(
                    "could not get prices for CPO: {}\nreason: {}",
                    data.cpo_name, error
                );
                Err(eyre::Error::msg(err_msg))
            }
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
                .get(self.build_url("v1/companies"))
                .query(&[("page[number]", page), ("page[size]", 100)])
                .send()
                .await?
                .error_for_status()?
                .json::<CompanyResponses>()
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
            .json::<serde_json::Value>()
            .await?;

        let json = json
            .pointer("/data/0/attributes/restricted_segments")
            .ok_or_else(|| eyre::Error::msg("wrong tariff details json response schema"))?;

        let details: Vec<TariffDetails> = serde_json::from_value(json.clone())?;

        let TariffsWithBlockingFee {
            tariff_id, cpo_id, ..
        } = body.data.context;

        let dimensions = details
            .iter()
            .filter(|item| item.dimension == DimenSion::Minute)
            .take(2)
            .collect::<Vec<_>>();

        let ac_dc_prices = dimensions
            .iter()
            .all(|item| item.charge_point_energy_type.is_none());

        if ac_dc_prices {
            let price = dimensions.first().map(|d| d.price).unwrap_or_default();
            let blocking_ac_tariff = TariffBlockingPrice {
                tariff_id: tariff_id,
                cpo_id: cpo_id,
                price,
                plug: ChargeType::AC,
            };
            return Ok(vec![
                blocking_ac_tariff.clone(),
                TariffBlockingPrice {
                    plug: ChargeType::DC,
                    ..blocking_ac_tariff
                },
            ]);
        }

        let ret_list = dimensions
            .iter()
            .filter_map(|item| {
                item.charge_point_energy_type
                    .map(|plug| TariffBlockingPrice {
                        tariff_id: tariff_id,
                        cpo_id: cpo_id,
                        price: item.price,
                        plug,
                    })
            })
            .collect::<Vec<_>>();
        Ok(ret_list)
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
            .map(|request| self.fetch_tariff_detail(request));

        let tariff_details = futures_util::future::try_join_all(requests)
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(tariff_details)
    }
}
