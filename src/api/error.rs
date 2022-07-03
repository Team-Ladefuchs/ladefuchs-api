use axum::{
    extract::rejection::PathRejection,
    http::{header::InvalidHeaderValue, StatusCode},
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
    #[error("Wrong username or password")]
    Login,
    #[error("Cookie has expired")]
    LoginTimeOut,
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
pub struct ErrorJson {
    pub status_code: u16,
    pub reason: String,
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
            ApiError::LoginTimeOut | ApiError::MissingToken | ApiError::Login => {
                StatusCode::UNAUTHORIZED
            }
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
