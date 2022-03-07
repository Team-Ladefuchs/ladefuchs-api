use std::{collections::HashMap, sync::Arc};

use crate::{
    config::Config,
    db::{self, charging, cpo, tarif::Tarif, vehicle::VehicleType},
    State,
};
use charging::Plug;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MSPApiResult {
    pub id: uuid::Uuid,
    pub attributes: MspAttributes,
    pub relationships: HashMap<String, HashMap<String, TarifJson>>,
}

impl MSPApiResult {
    pub fn into_tarif(&self, vehicle_id: i32, msp_id: i32) -> Tarif {
        let relationship_id = self
            .relationships
            .get("tariff")
            .and_then(|tarifs| tarifs.get("data"))
            .unwrap()
            .id;

        Tarif {
            relationship_id,
            vehicle_id,
            slug_name: self.attributes.tariff_name.clone(),
            monhtly_fee: self.attributes.total_monthly_fee,
            msp_id,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TarifJson {
    #[serde(rename = "type")]
    c_type: String,
    id: uuid::Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MspAttributes {
    pub provider: String,
    pub tariff_name: String,
    pub total_monthly_fee: f64,
    pub charge_point_prices: Vec<ChargePointPrice>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChargePointPrice {
    pub power: f64,
    pub plug: Plug,
    pub price: f64,
    pub blocking_fee_start: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResultWrapper {
    pub vehicle_id: i32,
    pub cpo_id: i32,
    pub msps: Vec<MSPApiResult>,
}

pub type AllChargePrices = Vec<ApiResultWrapper>;

pub async fn fetch_data(state: &State) -> Result<AllChargePrices, eyre::Error> {
    let mut connection = state.database_pool.acquire().await?;
    let api = Arc::new(ChargePriceAPI {
        config: state.config.clone(),
        vehicles: db::vehicle::get_vehicles(&mut connection).await?,
    });

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
        .filter_map(Result::ok)
        .flat_map(|item| item.into_iter())
        .collect();

    Ok(ret)
}

struct ChargePriceAPI {
    config: Config,
    vehicles: Vec<db::vehicle::Vehicle>,
}

impl ChargePriceAPI {
    pub async fn get_prices_for_cpo(&self, cpo: cpo::CPO) -> Result<AllChargePrices, eyre::Error> {
        self.do_api_call(self.build_reuest_body(cpo)).await
    }

    async fn do_api_call(&self, data: RequestPayload) -> Result<AllChargePrices, eyre::Error> {
        tracing::info!("price for cpo: {:?}", &data.cpo_name);
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_LANGUAGE, "en".parse().unwrap());
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
        );
        headers.insert("API-Key", self.config.charge_price_api_key.parse()?);
        let mut payload = HashMap::new();
        payload.insert("data", data.clone());
        let ret = reqwest::Client::new()
            .post(self.config.charge_price_api_url.clone())
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        let map: HashMap<String, Value> = ret.json().await?;
        // tracing::info!("{:?}", &map);
        if let Some(msps_values) = map.get("data") {
            let msps: Vec<MSPApiResult> = serde_json::from_value(msps_values.to_owned())?;
            // tracing::info!(
            //     "api json {}",
            //     serde_json::value::to_value(msps.to_owned())
            //         .unwrap()
            //         .to_string()
            // );

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
        } else {
            // TODO do better error report
            tracing::warn!("HHM not so good {:#?}", map);
            Err(eyre::Error::msg(format!(
                "Import whent wrong reponse: {:#?}",
                map
            )))
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

#[derive(Serialize, Debug, Clone)]
pub struct RequestPayload {
    #[serde(rename = "type")]
    r_type: &'static str,
    attributes: Attributes,
    relationships: HashMap<String, VehicleJson>,
    #[serde(skip)]
    cpo_name: String,
    #[serde(skip)]
    cpo_id: i32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            energy: 1,
            duration: 1,
            max_monthly_fees: 0.0,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
struct Attributes {
    data_adapter: &'static str,
    station: Station,
    options: Options,
}

#[derive(Serialize, Debug, Clone)]
struct Station {
    longitude: f32,
    latitude: f32,
    country: &'static str,
    network: uuid::Uuid,
    charge_points: Vec<ChargePoint>,
}

#[derive(Serialize, Debug, Clone)]
struct ChargePoint {
    power: u16,
    plug: Plug,
}

#[derive(Serialize, Debug, Clone)]
struct Options {
    energy: u32,
    duration: u32,
    max_monthly_fees: f32,
}

#[derive(Serialize, Debug, Clone)]
struct VehicleJson {
    data: VehicleData,
}

#[derive(Serialize, Debug, Clone)]
struct VehicleData {
    id: uuid::Uuid,
    #[serde(rename = "type")]
    c_type: VehicleType,
}
