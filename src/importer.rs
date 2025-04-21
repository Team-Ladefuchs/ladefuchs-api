// use chrono::offset::Utc;
// use eyre::Context;
// use sqlx::{Connection, PgConnection};
// use std::ops::Sub;

// use crate::{
//     db::{
//         self,
//         charge_price::save_alle_prices,
//         operator,
//         tariff::{save_tariffs, TariffContext},
//     },
//     slack::SlackClient,
//     state::State,
//     timer::Interval,
// };

// pub fn spawn_price_task(state: State, mut interval: Interval) -> tokio::task::JoinHandle<()> {
//     let duration = state.config.interval_minutes;
//     tokio::task::spawn(async move {
//         tokio::time::sleep(seconds(1)).await;
//         tracing::info!(
//             status = "Import task started",
//             interval = format!("{} minutes ⏰", duration.num_minutes())
//         );

//         loop {
//             interval.recv().await;
//             {
//                 tracing::info!(status = "Starting new import");
//                 match import_by_schedule(&state).await {
//                     Ok(_) => {
//                         tracing::info!(status = "Charge price import is done");
//                     }
//                     Err(error) => {
//                         if let Some(slack) = &state.slack {
//                             slack
//                                 .send_error_message(format!("while import prices: {}", error))
//                                 .await;
//                         }
//                         tracing::error!("Chargeprice API error, result={error}");
//                     }
//                 }
//                 tracing::info!(
//                     status = format!(
//                         "Next import from from chargeprice.app 🌐 in {} minutes ⏰",
//                         duration.num_minutes()
//                     )
//                 );
//             }
//         }
//     })
// }

// impl State {
    // async fn import_operators(&self, connection: &mut PgConnection) -> Result<(), eyre::Report> {
    //     tracing::info!(status = "Start import operators");

    //     let mut transaction = connection.begin().await?;
    //     let companies = self.charge_price_api.fetch_operator().await?;

    //     db::operator::insert_or_update_companies(&mut transaction, &companies).await?;

    //     tracing::info!(status = "fetched operators", operators = companies.len());

    //     transaction.commit().await?;

    //     let mut transaction_stations = connection.begin().await?;
    //     let charge_stations = self
    //         .charge_price_api
    //         .fetch_operator_charging_stations()
    //         .await?;

    //     tracing::info!(status = "import operators and statistics complete");
    //     db::operator::update_charge_stations_statistics(&mut transaction_stations, charge_stations)
    //         .await?;

    //     transaction_stations.commit().await?;

    //     Ok(())
    // }

//     pub async fn import_prices_and_operators(&self) -> Result<usize, eyre::Error> {
//         let Some(_lock) = self.lock() else {
//             tracing::info!("Skipped import because another import is in progress");
//             return Ok(0);
//         };

//         let mut connection = self.database_pool.acquire().await?;
//         self.import_operators(&mut connection)
//             .await
//             .with_context(|| "Error while import operator and charging stations statistic")?;

//         let mut operators =
//             operator::admin::get_with(&mut connection, operator::Filter::All).await?;

//         operators.sort_by(|a, b| b.standard.cmp(&a.standard));

//         tracing::info!(status = "fetch prices and tariffs");

//         let tariff_price_response = self
//             .charge_price_api
//             .fetch_all_tariff_prices(&operators)
//             .await;

//         let mut transaction = connection.begin().await?;

//         tracing::info!(
//             status = "save tariffs",
//             count = tariff_price_response.tariffs.len()
//         );

//         save_tariffs(TariffContext {
//             transaction: &mut transaction,
//             slack: &self.slack,
//             response: &tariff_price_response,
//         })
//         .await?;

//         transaction.commit().await?;

//         let prices_count = tariff_price_response.charge_prices.len();
//         tracing::info!(status = "Received prices", count = prices_count);

//         tracing::info!(status = "Writing charge prices to database");
//         let mut transaction_prices = connection.begin().await?;

//         if operators.len() == 1 {
//             db::charge_price::clear_by_operator(&mut transaction_prices, operators[0].id).await?;
//         } else {
//             db::charge_price::clear_all(&mut transaction_prices).await?;
//         }

//         if tariff_price_response.charge_prices.is_empty() {
//             transaction_prices.rollback().await?;
//             let msg = "Zero prices from Chargeprice received during last import. Current stored prices and tariffs will remain unchanged. Maybe the Chargeprice API is down.";
//             tracing::warn!(msg = msg);
//             let slack = &self.slack;
//             slack.send_warning_message(msg.to_string()).await;
//             return Ok(0);
//         }

//         save_alle_prices(&mut transaction_prices, tariff_price_response.charge_prices).await?;

//         transaction_prices.commit().await?;

//         let disabled_operators =
//             db::operator::get_standard_with_no_prices(&mut *connection).await?;

//         if disabled_operators.is_empty() {
//             return Ok(prices_count);
//         }

//         let message = format!(
//             "These standard CPOs have no prices:\n{}",
//             &disabled_operators.join(", "),
//         );
//         tracing::warn!(message);
//         if let Some(slack) = &self.slack {
//             slack.send_warning_message(message).await;
//         }

//         Ok(prices_count)
//     }
// }

// async fn import_by_schedule(state: &State) -> Result<(), eyre::Error> {
//     let mut connection = state.database_pool.acquire().await?;

//     let import_result = db::charge_price::last_import_context(&mut connection, None).await?;

//     if let Some(last_import) = import_result.last_import {
//         if Utc::now().sub(last_import).num_minutes() < 30 {
//             tracing::info!(
//                 status =
//                     "Skipping scheduled price import because last import was last than an 30 minutes ago"
//             );
//             return Ok(());
//         }
//     }

//     state.import_prices_and_operators().await?;

//     let tariff_count = db::tariff::get_count(&mut *connection).await?;
//     tracing::info!(status = "Check tariffs", "count" = tariff_count);

//     Ok(())
// }

pub const fn hours(h: u64) -> std::time::Duration {
    std::time::Duration::from_secs(3600 * h)
}

pub const fn seconds(s: u64) -> std::time::Duration {
    std::time::Duration::from_secs(s)
}
