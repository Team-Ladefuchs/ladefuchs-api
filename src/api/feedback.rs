use serde::{Deserialize, Serialize};

use crate::ladefuchs_db::plug::ChargeType;

pub mod v3 {

    use axum::{Extension, Json};
    use serde::Deserialize;

    use crate::{
        api,
        api::feedback::{Address, Coordinates},
        ladefuchs_db::{self, feedback::save, plug::ChargeType},
        state::State,
    };

    use super::{Feedback, FeedbackKind, LanguageCode, WrongPriceContext};

    #[derive(Deserialize, Debug)]
    pub struct FeedbackWithContextRequest {
        context: Context,
        request: RequestType,
    }

    #[derive(Deserialize, Debug)]
    #[serde(tag = "type", content = "attributes")]
    pub enum RequestType {
        #[serde(rename = "wrongPriceFeedback")]
        WrongPrice(WrongPriceAttributeRequest),
        #[serde(rename = "otherFeedback")]
        Other(OtherAttributeRequest),
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Context {
        pub operator_id: uuid::Uuid,
        pub tariff_id: uuid::Uuid,
        #[serde(default)]
        pub language: LanguageCode,
        pub email: Option<String>,
        pub payment_provider: Option<String>,
        pub address: Option<Address>,
        pub coordinates: Option<Coordinates>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct WrongPriceAttributeRequest {
        pub notes: String,
        pub displayed_price: f32,
        pub actual_price: f32, // (100): Either total price or price per kWh/minute. Whatever the user has at hand.
        pub charge_type: Option<ChargeType>,
    }
    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct OtherAttributeRequest {
        pub notes: String,
    }

    const MIN_NOTE_CHAR_LEN: usize = 11;

    pub async fn post_handler(
        Extension(state): Extension<State>,
        Json(payload): Json<FeedbackWithContextRequest>,
    ) -> Result<(), api::ApiError> {
        let mut connection = state.database_pool.acquire().await?;

        let Context {
            tariff_id,
            operator_id,
            language,
            address,
            coordinates,
            email,
            payment_provider,
        } = payload.context;

        let operator = ladefuchs_db::operator::get_by_pub_id(&mut connection, &operator_id).await?;

        let tariff = ladefuchs_db::tariff::get_by_public_id(&mut connection, &tariff_id)
            .await?
            .ok_or_else(|| api::ApiError::TariffNotFound(tariff_id))?;

        let feedback = match payload.request {
            RequestType::WrongPrice(wrong_price)
                if wrong_price.notes.len() > MIN_NOTE_CHAR_LEN
                    && wrong_price.actual_price != wrong_price.displayed_price =>
            {
                Some(Feedback {
                    tariff_id: tariff.id,
                    operator_id: operator.id,
                    notes: wrong_price.notes.clone(),
                    language,
                    context: Some(WrongPriceContext {
                        displayed_price: wrong_price.displayed_price,
                        actual_price: wrong_price.actual_price,
                        charge_type: wrong_price.charge_type,
                        address,
                        coordinates,
                        email,
                        payment_provider,
                    }),
                    kind: FeedbackKind::WrongPrice,
                })
            }
            RequestType::Other(other) if other.notes.len() > MIN_NOTE_CHAR_LEN => Some(Feedback {
                tariff_id: tariff.id,
                operator_id: operator.id,
                notes: other.notes.clone(),
                language,
                context: None,
                kind: FeedbackKind::Other,
            }),
            _ => None,
        };

        if let Some(feedback) = feedback {
            tracing::info!(
                status = "New Feedback",
                operator = operator.slug_name,
                tariff = tariff.slug_name,
                emp = tariff.provider_name,
                text = feedback.notes,
                context = ?feedback.context,
            );
            save(&mut connection, feedback).await?;
        }

        Ok(())
    }
}

#[derive(Debug, strum_macros::Display, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LanguageCode {
    #[strum(to_string = "de")]
    #[default]
    De,
}

#[derive(Debug, sqlx::Type, Copy, Clone)]
#[sqlx(type_name = "FeedbackKind")]
#[sqlx(rename_all = "snake_case")]
pub enum FeedbackKind {
    WrongPrice,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub street: String,
    pub postal_code: String,
    pub city: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    pub timestamp: f64,
}

#[derive(Debug)]
pub struct Feedback {
    pub notes: String,
    pub language: LanguageCode,
    pub tariff_id: i32,
    pub operator_id: i32,
    pub kind: FeedbackKind,
    pub context: Option<WrongPriceContext>,
}

#[derive(Debug, Serialize)]
pub struct WrongPriceContext {
    pub displayed_price: f32,
    pub actual_price: f32,
    pub charge_type: Option<ChargeType>,
    pub email: Option<String>,
    pub payment_provider: Option<String>,
    pub address: Option<Address>,
    pub coordinates: Option<Coordinates>,
}
