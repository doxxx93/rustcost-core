use axum::Json;

use crate::api::dto::ApiResponse;
use crate::errors::AppError;

/// Map a domain Result<T> into Json<ApiResponse<T>> with AppError mapping.
pub fn to_json<T: serde::Serialize>(
    result: anyhow::Result<T>,
) -> Result<Json<ApiResponse<T>>, AppError> {
    match result {
        Ok(v) => Ok(Json(ApiResponse::ok(v))),
        Err(_) => Err(AppError::InternalServerError),
    }
}
