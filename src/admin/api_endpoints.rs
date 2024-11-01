use axum::{
    extract::{Json, Path, Query},
    Extension,
};

use crate::{
    api::{
        app_metrics::admin::AppMetricsResponse,
        error::{self, ApiError},
        json, json_list,
        operator::v3::OperatorQueryFilter,
        ApiJson, ApiJsonList,
    },
    db::{
        self,
        banner::{banner_click_statistics, banner_click_summary, ClicksPerDay, ThgClickSummery},
        charge_price, image,
        operator::{self, admin},
        tariff,
    },
    importer,
    slack::{self, Emoji, SlackClient},
    state::State,
};

use super::jwt_auth::AdminUser;

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
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AppMetricQuery {
    days: u16,
}

pub async fn get_app_metrics(
    Extension(state): Extension<State>,
    Query(query): Query<AppMetricQuery>,
) -> Result<ApiJson<AppMetricsResponse>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;

    let metrics = AppMetricsResponse {
        usage_by_platform: db::app_metrics::admin::app_usage_number_by_platform(&mut connection, 0)
            .await?,
        usage_group_by_day: db::app_metrics::admin::app_usage_group_by_day(
            &mut connection,
            query.days.into(),
        )
        .await?,
        total_banner_impression: db::app_metrics::admin::banner_impression_last_days(
            &mut connection,
            0,
        )
        .await?,
    };
    Ok(json(metrics))
}

pub async fn get_operators(
    Extension(state): Extension<State>,
    filter: Query<OperatorQueryFilter>,
) -> Result<ApiJsonList<admin::Operator>, error::ApiError> {
    let mut connection = state.database_pool.acquire().await?;
    let operators = if filter.standard {
        db::operator::admin::get_with(&mut connection, operator::Filter::Enabled).await?
    } else {
        db::operator::admin::get_with(&mut connection, operator::Filter::All).await?
    };

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
            let msg = format!("Hi {},this CPO {:#?} has no image.\nI have some useful information:\nName Internal: {}\n{}", slack::MALIK, &operator.slug_name, &operator.name, url_str);
            slack
                .send_message(slack::TextMessage {
                    emoji: Some(Emoji::ElectricPlug),
                    text: msg,
                })
                .await;
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
                charge_price::last_import_context(&mut connection, Some(interval_time)).await?;
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
    admin_user: AdminUser,
    Extension(state): Extension<State>,
) -> Result<(), error::ApiError> {
    if state.is_import_locked() {
        return Err(ApiError::ImportInProgress);
    }

    tokio::task::spawn(async move {
        let slack = &state.slack;

        slack
            .send_message(slack::TextMessage {
                emoji: Some(Emoji::Dollar),
                text: format!(
                    "Manual price import was triggered by {}. This might take a few minutes.",
                    admin_user.username
                ),
            })
            .await;

        match state
            .import_prices_and_operators(importer::Mode::Manual)
            .await
        {
            Ok(prices_count) => {
                state.timer.restart().await;
                slack
                    .send_message(
                        slack::TextMessage { emoji: Some(Emoji::Dollar), text: format!("Manual price import finished successfully. It was triggered by {}. Fetched {} prices.", admin_user.username, prices_count)}
                    )
                    .await;
            }
            Err(err) => {
                slack
                    .send_message(slack::TextMessage {
                        emoji: Some(Emoji::Warning),
                        text: format!("Error occurred during manual import: {}", err),
                    })
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
