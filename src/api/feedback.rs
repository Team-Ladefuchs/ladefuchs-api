pub mod v3 {
    use axum::{Extension, Json};
    use serde::Deserialize;

    use crate::{
        api,
        charge_price_api::request::{
            feedback::{self, LanguageCode},
            DataWrapper,
        },
        db,
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
        pub email: String,
        #[serde(default)]
        pub language: LanguageCode,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct WrongPriceAttributeRequest {
        pub notes: String,
        pub displayed_price: f32,
        pub actual_price: f32, // (100): Either total price or price per kWh/minute. Whatever the user has at hand.
    }
    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct OtherAttributeRequest {
        pub notes: String,
    }

    pub async fn post_handler(
        Extension(state): Extension<State>,
        Json(payload): Json<FeedbackWithContextRequest>,
    ) -> Result<(), api::ApiError> {
        let mut connection = state.database_pool.acquire().await?;

        let Context {
            tariff_id,
            operator_id,
            email,
            language,
        } = payload.context;

        let operator =
            db::operator::get_by_pub_id_or_name(&mut connection, &operator_id.to_string()).await?;

        let tariff = db::tariff::get_by_public_id(&mut connection, &tariff_id)
            .await?
            .ok_or_else(|| api::ApiError::TariffNotFound(tariff_id))?;

        let cp_context = format!(
            "[cpo: {}, tariff: {}, Ladefuchs App]",
            operator.slug_name, tariff.slug_name
        );

        let attributes = match payload.request {
            RequestType::WrongPrice(wrong_price) => {
                feedback::TypeAttribute::WrongPrice(feedback::WrongPriceAttribute {
                    context: cp_context,
                    tariff: tariff.slug_name,
                    cpo: operator.slug_name,
                    poi_link: "",
                    email,
                    notes: wrong_price.notes,
                    language,
                    displayed_price: wrong_price.displayed_price.to_string(),
                    actual_price: wrong_price.actual_price.to_string(),
                })
            }
            RequestType::Other(other) => feedback::TypeAttribute::Other(feedback::OtherAttribute {
                email,
                notes: other.notes,
                language,
                context: cp_context,
            }),
        };

        let cp_feedback_request = DataWrapper { data: attributes };

        state
            .charge_price_api
            .send_feedback(&cp_feedback_request)
            .await?;
        Ok(())
    }
}
