pub mod v3 {
    use std::usize;

    use axum::{Extension, Json};
    use serde::Deserialize;

    use crate::{
        api,
        charge_price_api::request::feedback::{self, FeedbackKind, LanguageCode},
        db::{self, feedback::save, plug::ChargeType},
        state::State,
    };

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
        } = payload.context;

        let operator = db::operator::get_by_pub_id(&mut connection, &operator_id).await?;

        let tariff = db::tariff::get_by_public_id(&mut connection, &tariff_id)
            .await?
            .ok_or_else(|| api::ApiError::TariffNotFound(tariff_id))?;

        let feedback = match payload.request {
            RequestType::WrongPrice(wrong_price) if wrong_price.notes.len() > MIN_NOTE_CHAR_LEN => {
                Some(feedback::Feedback {
                    tariff_id: tariff.id,
                    operator_id: operator.id,
                    notes: wrong_price.notes.clone(),
                    language,
                    context: Some(feedback::WrongPriceContext {
                        displayed_price: wrong_price.displayed_price,
                        actual_price: wrong_price.actual_price,
                        charge_type: wrong_price.charge_type,
                    }),
                    kind: FeedbackKind::WrongPrice,
                })
            }
            RequestType::Other(other) if other.notes.len() > MIN_NOTE_CHAR_LEN => {
                Some(feedback::Feedback {
                    tariff_id: tariff.id,
                    operator_id: operator.id,
                    notes: other.notes.clone(),
                    language,
                    context: None,
                    kind: FeedbackKind::WrongPrice,
                })
            }
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
