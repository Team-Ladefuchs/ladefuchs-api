use super::{
    request::{
        Attributes, ChargePoint, Options, RequestPayload, Station, VehicleData, VehicleJson,
    },
    response::AllChargePrices,
};
use crate::{
    api::operator::Filter,
    charge_price_api::{
        request::Relationship,
        response::{ApiResultWrapper, ResponseError},
    },
    config::Config,
    db::{self, cpo},
    State,
};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

pub async fn fetch_data(state: &State) -> Result<AllChargePrices, eyre::Error> {
    let mut connection = state.database_pool.acquire().await?;
    let vehicles = db::vehicle::get_vehicles(&mut connection).await?;
    let api = Arc::new(ChargePriceAPI::new(&state.config, vehicles)?);

    let tasks = cpo::get_with(&mut state.database_pool.acquire().await?, Filter::Enabled)
        .await?
        .into_iter()
        .flat_map(|cpo| {
            tracing::info!("fetching data for CPO: {:?}", cpo.name);
            api.build_request_body(&cpo)
        })
        .map(|request| {
            let api = api.clone();
            tokio::task::spawn(async move { api.do_api_call(&request).await })
        })
        .collect::<Vec<_>>();

    let ret = futures_util::future::try_join_all(tasks)
        .await?
        .into_iter()
        // TODO log if result was bad
        .filter_map(Result::ok)
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
                    let errs: Vec<ResponseError> = serde_json::from_value(err.to_owned())?;

                    let err_msg = format!(
                        "could not get prices for CPO: {}, status: {}",
                        payload.cpo_name, status_code
                    );
                    tracing::error!(err_msg=%err_msg, errors=?errs);
                    Err(eyre::Error::msg(err_msg))
                }
                None => Err(unknown_response(&payload.cpo_name)),
            };
        }
        let json: HashMap<String, Value> = ret.json().await?;
        match json.get("data") {
            Some(msps_values) => Ok(ApiResultWrapper {
                vehicle_id: payload.vehicle_id,
                cpo_id: payload.cpo_id,
                msps: serde_json::from_value(msps_values.to_owned())?,
            }),
            None => Err(unknown_response(&payload.cpo_name)),
        }
    }

    fn build_request_body(&self, cpo: &cpo::CPO) -> Vec<RequestPayload> {
        let charge_points = cpo
            .supported_types
            .iter()
            .map(|(plug, meta)| ChargePoint {
                power: meta.power as u16,
                plug: plug.clone(),
            })
            .collect::<Vec<_>>();

        let requests = self
            .vehicles
            .clone()
            .into_iter()
            .map(|vehicle| {
                let vehicle_json = match vehicle.vehicle_type {
                    db::vehicle::VehicleType::Empty => None,
                    _ => Some(VehicleJson {
                        data: VehicleData {
                            id: vehicle.uuid,
                            c_type: vehicle.vehicle_type,
                        },
                    }),
                };
                RequestPayload {
                    cpo_id: cpo.id,
                    cpo_name: cpo.slug_name.clone(),
                    r_type: "charge_price_request",
                    attributes: Attributes {
                        station: Station {
                            longitude: 0.0,
                            latitude: 0.0,
                            country: "DE",
                            network: cpo.network,
                            charge_points: charge_points.clone(),
                        },
                        data_adapter: "chargeprice",
                        options: Options::default(),
                    },
                    vehicle_id: vehicle.id,
                    relationships: Relationship {
                        vehicle: vehicle_json,
                    },
                }
            })
            .collect();
        tracing::debug!("{:?}", request = &requests);
        requests
    }
}

fn unknown_response(cpo_name: &str) -> eyre::Error {
    eyre::Error::msg(format!("Unkown API response for CPO: {}", cpo_name))
}
