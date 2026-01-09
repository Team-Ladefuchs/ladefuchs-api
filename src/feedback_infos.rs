use chrono::{Timelike, Utc};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

use crate::{api::feedback::FeedbackKind, slack, state::State};

pub async fn schedule_feedbacks(scheduler: &JobScheduler, state: State) -> Result<(), eyre::Error> {
    scheduler.add(Job::new_async(
        "0 0 18 * * *",
        move |uuid: uuid::Uuid, mut lock| {
            Box::pin({
                let state_value = state.clone();

                async move {
                    tracing::info!(timestamp = %Utc::now().to_rfc3339(), "trigger feedback infos");

                    if let Err(err) = send_feedback_infos(&state_value).await {
                        warn!("Could not send feedback infos: {}", err);
                    }

                    tracing::info!(timestamp = %Utc::now().to_rfc3339(), "finished feedback infos");

                    let next_tick = lock.next_tick_for_job(uuid).await;
                    match next_tick {
                        Ok(Some(ts)) => info!("Next time job is {:?}", ts),
                        _ => error!("Could not get next tick for 7s job"),
                    }
                }
            })
        },
    )?).await?;

    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
struct KindCountRow {
    kind: FeedbackKind,
    cnt: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct PriceOffenderRow {
    slug_name: String,
    cnt: i64,
}

async fn send_feedback_infos(state: &State) -> Result<(), eyre::Error> {
    let Some(slack) = state.slack.as_ref() else {
        info!("No slack configured, skipping feedback info sending");
        return Ok(());
    };

    let now = Utc::now();

    let last_run = now
        .with_hour(18)
        .and_then(|dt| dt.with_minute(0))
        .and_then(|dt| dt.with_second(0))
        .and_then(|dt| dt.with_nanosecond(0))
        .map(|dt| dt - chrono::Duration::days(1))
        .ok_or_else(|| eyre::Error::msg("Error forming last run time"))?;

    let no_feedbacks_since_last_run: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM feedback WHERE updated >= $1",
        last_run
    )
    .fetch_one(&state.database_pool)
    .await?
    .unwrap_or(0);

    if no_feedbacks_since_last_run == 0 {
        info!(
            "No feedbacks since last run at {}, skipping feedback info sending",
            last_run
        );

        return Ok(());
    };

    let top_five_kinds: Vec<KindCountRow> = sqlx::query_as(
        r#"
        SELECT kind , COUNT(*) as "cnt"
        FROM feedback
        WHERE updated >= $1
        GROUP BY kind
        ORDER BY cnt DESC
		LIMIT 5
        "#,
    )
    .bind(last_run)
    .fetch_all(&state.database_pool)
    .await?;

    let top_five_price_offending_operators: Vec<PriceOffenderRow> = sqlx::query_as(
        r#"
        SELECT slug_name, COUNT(*) as "cnt"
        FROM feedback
        INNER JOIN operator ON feedback.operator_id = operator.id
        WHERE
            feedback.updated >= $1
            AND feedback.kind = 'wrong_price'
        GROUP BY slug_name
        ORDER BY cnt DESC
        LIMIT 5
        "#,
    )
    .bind(last_run)
    .fetch_all(&state.database_pool)
    .await?;

    let top_five_price_offending_tariffs: Vec<PriceOffenderRow> = sqlx::query_as(
        r#"
        SELECT slug_name, COUNT(*) as "cnt"
        FROM feedback
        INNER JOIN tariff ON feedback.tariff_id = tariff.id
        WHERE
            feedback.updated >= $1
            AND feedback.kind = 'wrong_price'
        GROUP BY slug_name
        ORDER BY cnt DESC
        LIMIT 5
        "#,
    )
    .bind(last_run)
    .fetch_all(&state.database_pool)
    .await?;

    let local_time = last_run
        .with_timezone(&chrono::Local)
        .format("%d.%m.%Y um %H:%M Uhr")
        .to_string();

    let feedback_types_str = top_five_kinds
        .iter()
        .map(|row| format!("- {}: {}", kind_to_str(row.kind), row.cnt))
        .collect::<Vec<String>>()
        .join("\n");

    let top_five_price_offending_operators_str = top_five_price_offending_operators
        .iter()
        .map(|row| format!("- {}: {}", row.slug_name, row.cnt))
        .collect::<Vec<String>>()
        .join("\n");

    let top_five_price_offending_tariffs_str = top_five_price_offending_tariffs
        .iter()
        .map(|row| format!("- {}: {}", row.slug_name, row.cnt))
        .collect::<Vec<String>>()
        .join("\n");

    let msg = format!(
        r#"
# Feedback-Zusammenfassung seit dem letzten Lauf ({local_time})

Anzahl eingegangener Feedbacks: {no_feedbacks_since_last_run}

## :moneybag: Top 5 Betreiber mit falschen Preisen:

{top_five_price_offending_operators_str}

## :fire: Top 5 Tarife mit falschen Preisen:

{top_five_price_offending_tariffs_str}

## :low_battery: Top 5 Feedback-Arten:

{feedback_types_str}
"#
    );

    slack
        .send(slack::TextMessage {
            emoji: Some(slack::Emoji::Dollar),
            text: msg,
            markdown: true,
        })
        .await;

    Ok(())
}

fn kind_to_str(kind: FeedbackKind) -> &'static str {
    match kind {
        FeedbackKind::WrongPrice => "Falscher Preis",
        FeedbackKind::Other => "Sonstiges",
    }
}
