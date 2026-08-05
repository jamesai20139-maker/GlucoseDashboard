use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotConfigured(String),
    #[error("{0}")]
    Sync(String),
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    Internal(String),
}

#[derive(Serialize)]
pub struct ErrorBody {
    pub status: &'static str,
    pub code: &'static str,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::NotConfigured(_) => (StatusCode::PRECONDITION_REQUIRED, "NOT_CONFIGURED"),
            Self::Sync(_) => (StatusCode::BAD_GATEWAY, "SYNC_FAILED"),
            Self::Invalid(_) => (StatusCode::BAD_REQUEST, "INVALID_REQUEST"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };
        (
            status,
            Json(ErrorBody {
                status: "failed",
                code,
                message: self.to_string(),
            }),
        )
            .into_response()
    }
}
