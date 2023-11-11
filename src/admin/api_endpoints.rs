use axum::{
    extract::{Json, Path},
    Extension,
};

use crate::{
    api::{
        error::{self, ApiError},
        json, json_list, ApiJson, ApiJsonList,
    },
    db::{
        self,
        banner::{banner_click_statistics, banner_click_summary, ClicksPerDay, ThgClickSummery},
        charge_price, image,
        operator::{self, admin},
        tariff::{self},
    },
    importer,
    slack::{self, Emoji, SlackClient},
    state::State,
};

pub async fn get_all_tariffs(
    Extension(state): Extension<State>,
) -> Result<ApiJsonList<tariff::admin::TariffIntern>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let tariffs = tariff::admin::get_all(&mut connection).await?;

    Ok(json_list(tariffs))
}

pub async fn get_banner_chart_data(
    Extension(state): Extension<State>,
    Path((days, link_id)): Path<(i32, i32)>,
) -> Result<ApiJsonList<ClicksPerDay>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let clicks = banner_click_statistics(&mut connection, days, link_id).await?;
    Ok(json_list(clicks))
}

pub async fn get_banner_statistics(
    Extension(state): Extension<State>,
    Path(link_id): Path<i32>,
) -> Result<ApiJson<ThgClickSummery>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let summary = banner_click_summary(&mut connection, link_id).await?;
    Ok(json(summary))
}

pub async fn get_all_standard_operators(
    Extension(state): Extension<State>,
) -> Result<ApiJsonList<admin::Operator>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let operators =
        db::operator::admin::get_with(&mut connection, operator::Filter::Enabled).await?;

    Ok(json_list(operators))
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CpoSearchRequest {
    query: String,
}

pub async fn operator_search(
    Extension(state): Extension<State>,
    Json(request): Json<CpoSearchRequest>,
) -> Result<ApiJsonList<admin::Operator>, error::ApiError> {
    if request.query.is_empty() {
        Ok(json(vec![]))
    } else {
        let mut connection = state.database_pool.acquire().await?;
        let result = operator::search(&mut connection, &request.query).await?;
        Ok(json(result))
    }
}

pub async fn patch_operator(
    Extension(state): Extension<State>,
    Json(mut operator): Json<admin::Operator>,
) -> Result<ApiJson<admin::Operator>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    operator.update(&mut connection).await?;

    match operator.image {
        Some(image_id) => {
            if let Err(error) = image::update_image_file_name(
                &mut connection,
                &operator.name,
                image_id,
                Some("cpo"),
            )
            .await
            {
                tracing::error!(
                    operator_id = operator.id,
                    internal_name = operator.name,
                    slug_name = operator.slug_name,
                    image_id = image_id,
                    %error,
                    "Could update internal operator name",
                )
            }
        }
        None => {
            let slack = &state.slack;
            let url_str = operator
                .url
                .as_ref()
                .and_then(|s| Some(s.as_str()))
                .unwrap_or_default();
            let msg = format!("Hi {}, there is CPO {:#?} has no image.\nI have some useful information:\nName Internal: {}\n{}", slack::MALIK, &operator.slug_name, &operator.name, url_str);
            slack.send(Some(Emoji::ElectricPlug), &msg).await;
        }
    }

    Ok(json(operator))
}

pub async fn last_import(
    Extension(state): Extension<State>,
) -> Result<ApiJson<charge_price::admin::AdminImport>, error::ApiError> {
    let status = charge_price::admin::ImportStatus::from(state.is_import_locked());
    let import_result = match status {
        charge_price::admin::ImportStatus::Waiting => {
            let mut connection = state.database_pool.acquire().await?;
            let interval_time = state.timer.next().await?;
            let import_result =
                charge_price::import_metadata(&mut connection, Some(interval_time)).await?;
            Some(import_result)
        }
        charge_price::admin::ImportStatus::InProgress => None,
    };

    Ok(json(charge_price::admin::AdminImport {
        status,
        import_result,
    }))
}

pub async fn trigger_manual_import(
    Extension(state): Extension<State>,
) -> Result<(), error::ApiError> {
    if state.is_import_locked() {
        return Err(ApiError::ImportInProgress);
    }

    let mut connection: sqlx::pool::PoolConnection<sqlx::Postgres> =
        state.database_pool.acquire().await?;
    let operator_list = operator::admin::get_with(&mut connection, operator::Filter::All).await?;

    tokio::task::spawn(async move {
        let slack = &state.slack;

        match state
            .import_prices(&mut connection, importer::Mode::Manual, &operator_list)
            .await
        {
            Ok(prices_count) => {
                slack
                    .send(
                        None,
                        &format!("Manual import was successful. Prices: {}", prices_count),
                    )
                    .await;
            }
            Err(err) => {
                slack
                    .send(
                        None,
                        &format!("Error occurred during manual import: {}", err),
                    )
                    .await;
            }
        };
    });

    Ok(())
}

pub async fn patch_tariff(
    Extension(state): Extension<State>,
    Json(payload): Json<tariff::admin::UpdateTariffInternal>,
) -> Result<(), error::ApiError> {
    let mut connection: sqlx::pool::PoolConnection<sqlx::Postgres> =
        state.database_pool.acquire().await?;
    tariff::admin::update_partial(&mut connection, &payload).await?;
    Ok(())
}
