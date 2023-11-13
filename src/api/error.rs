use axum::{
    extract::rejection,
    http::{header::InvalidHeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::OutOfRangeError;
#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("internal server error")]
    General(eyre::Error),
    #[error("Import was not successfully: {0}")]
    Import(#[from] eyre::Error),
    #[error("state is not been set")]
    State,
    #[error("{0}")]
    PathExtractor(#[from] rejection::PathRejection),
    #[error(transparent)]
    QueryExtractor(#[from] rejection::QueryRejection),
    #[error("wrong authorization token got: {0}")]
    WrongToken(String),
    #[error("missing authorization token")]
    MissingToken,
    #[error("resource not found")]
    NotFound,
    #[error("{0} not found")]
    AffilateNotFound(String),
    #[error("operator: {0} does not exists")]
    OperatorNotFound(String),
    #[error("wrong username or password")]
    Login,
    #[error("cookie may has expired")]
    LoginTimeOut,
    #[error("An import is already in progress")]
    ImportInProgress,
    #[error(transparent)]
    JsonRejection(#[from] rejection::JsonRejection),
}

impl From<std::io::Error> for ApiError {
    fn from(_err: std::io::Error) -> Self {
        Self::NotFound
    }
}

impl From<InvalidHeaderValue> for ApiError {
    fn from(err: InvalidHeaderValue) -> Self {
        Self::General(eyre::Error::from(err))
    }
}

impl From<OutOfRangeError> for ApiError {
    fn from(err: OutOfRangeError) -> Self {
        Self::General(eyre::Error::from(err))
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        Self::General(eyre::Error::from(err))
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorJson {
    pub status_code: u16,
    pub reason: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::General(ref err) | ApiError::Import(ref err) => {
                tracing::error!(server_error =?err);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            ApiError::State => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::PathExtractor(_)
            | ApiError::MissingToken
            | ApiError::QueryExtractor(_)
            | ApiError::JsonRejection(_) => StatusCode::BAD_REQUEST,
            ApiError::LoginTimeOut | ApiError::Login | ApiError::WrongToken(_) => {
                StatusCode::UNAUTHORIZED
            }
            ApiError::NotFound | ApiError::OperatorNotFound(_) | ApiError::AffilateNotFound(_) => {
                StatusCode::NOT_FOUND
            }
            ApiError::ImportInProgress => StatusCode::CONFLICT,
        };
        let msg = self.to_string();

        if status != StatusCode::NOT_FOUND {
            tracing::info!(error= %self, status=%status, request_error=%msg);
        } else {
            tracing::debug!(error= %self, status=%status, request_error=%msg);
        }

        (
            status,
            Json(ErrorJson {
                status_code: status.as_u16(),
                reason: msg,
            }),
        )
            .into_response()
    }
}
