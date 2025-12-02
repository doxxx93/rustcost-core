use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;
use crate::api::dto::ApiResponse;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Body parsing error: {0}")]
    BodyParsingError(String),

    #[error("K8s API error: {0}")]
    K8sApiError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Not Resync: {0}")]
    NotResynced(String),
}

/// Helper for mapping any unknown error into internal error
pub fn internal_error<E: ToString>(err: E) -> AppError {
    AppError::InternalServerError(err.to_string())
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        // Choose status codes per variant
        let status = match self {
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BodyParsingError(_) => StatusCode::BAD_REQUEST,
            AppError::K8sApiError(_) => StatusCode::BAD_GATEWAY,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::NotResynced(_) => StatusCode::SERVICE_UNAVAILABLE,
        };

        // Extract error components
        let (code, msg) = match &self {
            AppError::InternalServerError(m) => ("InternalServerError", m.clone()),
            AppError::BodyParsingError(m) => ("BodyParsingError", m.clone()),
            AppError::K8sApiError(m) => ("K8sApiError", m.clone()),
            AppError::DatabaseError(m) => ("DatabaseError", m.clone()),
            AppError::NotFound(m) => ("NotFound", m.clone()),
            AppError::NotResynced(m) => ("NotResynced", m.clone()),
        };

        // Use your standardized ApiResponse
        let body = Json(ApiResponse::<()>::err_with_code(code, msg));

        (status, body).into_response()
    }
}
