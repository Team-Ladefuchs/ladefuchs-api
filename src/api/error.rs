use axum::{
    extract::rejection::PathRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("internal server error")]
    General(eyre::Error),
    #[error("state is not been set")]
    State,
    #[error("{0}")]
    PathExtractor(PathRejection),
    #[error("wrong authorization token got: `{0}`")]
    WrongToken(String),
    #[error("missing authorization token")]
    MissingToken,
    #[error("resource not found")]
    NotFound,
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        Self::General(eyre::Error::from(err))
    }
}
impl From<PathRejection> for ApiError {
    fn from(err: PathRejection) -> Self {
        Self::PathExtractor(err)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct ErrorJson {
    status_code: u16,
    reason: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::General(ref err) => {
                tracing::error!(server_error =%err);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            ApiError::State => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::WrongToken(_) | ApiError::PathExtractor(_) => StatusCode::BAD_REQUEST,
            ApiError::MissingToken => StatusCode::UNAUTHORIZED,
            ApiError::NotFound => StatusCode::NOT_FOUND,
        };
        let msg = self.to_string();
        tracing::warn!(request_error =%msg);

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
