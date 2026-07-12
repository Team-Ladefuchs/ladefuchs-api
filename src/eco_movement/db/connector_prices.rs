use super::*;
use crate::eco_movement::api::response::price::ConnectorPrice;

#[derive(Debug)]
struct PriceContext<'a> {
    location_id: uuid::Uuid,
    pricing_id: String,
    evse_uid: &'a str,
    connector_id: &'a str,
}

pub async fn save_multiple(
    connection: &mut PgConnection,
    connector_prices: Vec<ConnectorPrice>,
) -> Result<(), sqlx::Error> {
    for connector_price in connector_prices {
        save(connection, connector_price).await?;
    }

    Ok(())
}

pub async fn price_exists(connection: &mut PgConnection, price_id: &str) -> Option<String> {
    sqlx::query_file_scalar!("sql/insert/eco_movement/price_by_id.sql", price_id)
        .fetch_optional(&mut *connection)
        .await
        .ok()
        .flatten()
}

async fn connector_price_exists(
    connection: &mut PgConnection,
    context: &PriceContext<'_>,
) -> Result<bool, sqlx::Error> {
    sqlx::query_file_scalar!(
        "sql/get/eco_movement/connector_price_exsists.sql",
        context.location_id,
        context.pricing_id,
        context.evse_uid,
        context.connector_id
    )
    .fetch_one(&mut *connection)
    .await
}

async fn save(
    connection: &mut PgConnection,
    connector_price: ConnectorPrice,
) -> Result<(), sqlx::Error> {
    if let (Some(location_id), Some(connector_id)) = (
        location::location_exists(connection, &connector_price.location_id).await,
        connector::connector_exists(
            connection,
            (&connector_price.connector_id, &connector_price.evse_uid),
        )
        .await,
    ) {
        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO eco_movement.connector_price (location_id, pricing_id, evse_uid, connector_id)",
        );

        let mut price_queries = Vec::with_capacity(connector_price.pricing_ids.len());

        tracing::debug!("build price query start");

        for pricing_id in connector_price.pricing_ids {
            if let Some(price_id) = price_exists(connection, &pricing_id).await {
                let price_context = PriceContext {
                    location_id,
                    evse_uid: &connector_price.evse_uid,
                    pricing_id: price_id,
                    connector_id: &connector_id,
                };

                if connector_price_exists(connection, &price_context).await? {
                    continue;
                }

                price_queries.push(price_context);
            }
        }

        tracing::debug!(len = price_queries.len(), "build price query done");

        if price_queries.is_empty() {
            return Ok(());
        }

        query_builder.push_values(price_queries, |mut builder, new_price| {
            builder
                .push_bind(new_price.location_id)
                .push_bind(new_price.pricing_id)
                .push_bind(new_price.evse_uid)
                .push_bind(new_price.connector_id);
        });

        query_builder.build().execute(connection).await?;
        tracing::debug!("insert price done");
    }
    Ok(())
}
