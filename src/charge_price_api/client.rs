use std::{collections::HashMap, sync::Arc};

use crate::{
    charge_price_api::response::{ApiResultWrapper, MSPApiResult, ResponseError},
    config::Config,
    db::{self, cpo},
    State,
};

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE};
use serde_json::Value;

use super::{
    request::{
        Attributes, ChargePoint, Options, RequestPayload, Station, VehicleData, VehicleJson,
    },
    response::AllChargePrices,
};

pub async fn fetch_data(state: &State) -> Result<AllChargePrices, eyre::Error> {
    let mut connection = state.database_pool.acquire().await?;
    let vehicles = db::vehicle::get_vehicles(&mut connection).await?;
    let api = Arc::new(ChargePriceAPI::new(&state.config, vehicles)?);

    let tasks = cpo::get_all(&state.database_pool)
        .await?
        .into_iter()
        .map(|cpo| {
            let api = api.clone();
            tokio::task::spawn(async move { api.get_prices_for_cpo(cpo).await })
        })
        .collect::<Vec<_>>();

    let ret = futures_util::future::try_join_all(tasks)
        .await?
        .into_iter()
        // TODO log if result was bad
        // .filter_map(Result::ok)
        .map(|a| a.unwrap())
        .flat_map(|item| item.into_iter())
        .collect();

    Ok(ret)
}

struct ChargePriceAPI {
    client: reqwest::Client,
    vehicles: Vec<db::vehicle::Vehicle>,
    api_url: url::Url,
}

impl ChargePriceAPI {
    fn new(config: &Config, vehicles: Vec<db::vehicle::Vehicle>) -> Result<Self, eyre::Error> {
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
            vehicles,
            api_url: url::Url::from(config.charge_price_api_url.clone()),
        })
    }

    async fn get_prices_for_cpo(&self, cpo: cpo::CPO) -> Result<AllChargePrices, eyre::Error> {
        let data = self.build_reuest_body(cpo);

        tracing::info!("fetching data for CPO: {:?}", &data.cpo_name);

        let mut payload = HashMap::new();
        payload.insert("data", data.clone());
        let ret = self
            .client
            .post(self.api_url.clone())
            .json(&payload)
            .send()
            .await?;

        let status_code = ret.status();
        if status_code.ne(&reqwest::StatusCode::OK) {
            let json: HashMap<String, Value> = ret.json().await?;
            return match json.get("errors") {
                Some(err) => {
                    let errs: Vec<ResponseError> = serde_json::from_value(err.to_owned())?;

                    let err_msg = format!(
                        "could not get prices for CPO: {}, status: {}",
                        data.cpo_name, status_code
                    );
                    tracing::error!(err_msg=%err_msg, errors=?errs);
                    Err(eyre::Error::msg(err_msg))
                }
                None => Err(unkown_response(&data.cpo_name)),
            };
        }
        let json: HashMap<String, Value> = ret.json().await?;
        // tracing::info!("{:?}", &map);
        match json.get("data") {
            Some(msps_values) => {
                let msps: Vec<MSPApiResult> = serde_json::from_value(msps_values.to_owned())?;
                let results_by_vehicle = self
                    .vehicles
                    .iter()
                    .map(|vehicle| ApiResultWrapper {
                        vehicle_id: vehicle.id,
                        cpo_id: data.cpo_id,
                        msps: msps.clone(),
                    })
                    .collect::<Vec<ApiResultWrapper>>();
                Ok(results_by_vehicle)
            }
            None => Err(unkown_response(&data.cpo_name)),
        }
    }

    fn build_reuest_body(&self, cpo: cpo::CPO) -> RequestPayload {
        let charge_points = cpo
            .charge_map
            .iter()
            .map(|(plug, meta)| ChargePoint {
                power: meta.power as u16,
                plug: plug.clone(),
            })
            .collect();

        let relationships = self
            .vehicles
            .iter()
            .map(|vehicle| {
                (
                    vehicle.name.to_owned().to_string(),
                    VehicleJson {
                        data: VehicleData {
                            id: vehicle.uuid,
                            c_type: vehicle.vehicle_type,
                        },
                    },
                )
            })
            .collect();

        RequestPayload {
            cpo_id: cpo.id,
            cpo_name: cpo.slug_name,
            r_type: "charge_price_request",
            attributes: Attributes {
                station: Station {
                    longitude: 0.0,
                    latitude: 0.0,
                    country: "DE",
                    network: cpo.network,
                    charge_points,
                },
                data_adapter: "chargeprice",
                options: Options::default(),
            },
            relationships,
        }
    }
}

fn unkown_response(cpo_name: &str) -> eyre::Error {
    eyre::Error::msg(format!("Unkown API response for CPO: {}", cpo_name))
}
