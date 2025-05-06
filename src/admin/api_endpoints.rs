use axum::{
    Extension,
    extract::{Json, Path, Query},
};
use sqlx::Acquire;
use tracing::info;

use crate::{
    api::{
        ApiJson, ApiJsonList,
        app_metrics::admin::AppMetricsResponse,
        error::{self, ApiError},
        json, json_list,
        operator::v3::OperatorQueryFilter,
    },
    ladefuchs_db::{
        self,
        banner::{ClicksPerDay, ThgClickSummery, banner_click_statistics, banner_click_summary},
        image,
        operator::{self, admin},
        price, tariff,
    },
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
        usage_by_platform: ladefuchs_db::app_metrics::admin::app_usage_number_by_platform(
            &mut connection,
            0,
        )
        .await?,
        usage_group_by_day: ladefuchs_db::app_metrics::admin::app_usage_group_by_day(
            &mut connection,
            query.days.into(),
        )
        .await?,
        total_banner_impression: ladefuchs_db::app_metrics::admin::banner_impression_last_days(
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
        ladefuchs_db::operator::admin::get_with(&mut connection, operator::Filter::Enabled).await?
    } else {
        ladefuchs_db::operator::admin::get_with(&mut connection, operator::Filter::All).await?
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
    let mut transaction: sqlx::Transaction<sqlx::Postgres> = connection.begin().await?;
    operator.update(&mut transaction).await?;

    transaction.commit().await?;

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
            let url_str = operator.url.as_deref().unwrap_or_default();
            let msg = format!(
                "Hi {},this CPO {:#?} has no image.\nI have some useful information:\nName Internal: {}\n{}",
                slack::MALIK,
                &operator.slug_name,
                &operator.name,
                url_str
            );
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
) -> Result<ApiJson<price::admin::AdminImport>, error::ApiError> {
    let status = price::admin::ImportStatus::from(state.is_import_locked());
    let import_result = match status {
        price::admin::ImportStatus::Waiting => {
            let mut connection = state.database_pool.acquire().await?;
            let import_result = price::last_import_context(&mut connection, None).await?;
            Some(import_result)
        }
        price::admin::ImportStatus::InProgress => None,
    };

    Ok(json(price::admin::AdminImport {
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

    let slack = &state.slack;

    slack
        .send_message(slack::TextMessage {
            emoji: Some(Emoji::Dollar),
            text: format!(
                "Manual price import was triggered by {}. Nice Try :D",
                admin_user.username
            ),
        })
        .await;

    info!("ingore manuel import");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    Ok(())
}

pub async fn patch_tariff(
    Extension(state): Extension<State>,
    Json(payload): Json<tariff::admin::UpdateTariffInternal>,
) -> Result<(), error::ApiError> {
    let mut connection: sqlx::pool::PoolConnection<sqlx::Postgres> =
        state.database_pool.acquire().await?;
    let mut transaction: sqlx::Transaction<'_, sqlx::Postgres> = connection.begin().await?;
    tariff::admin::update_partial(&mut transaction, &payload).await?;
    transaction.commit().await?;

    Ok(())
}
