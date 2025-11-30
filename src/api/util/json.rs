use anyhow::Result;
use axum::Json;

use crate::api::dto::ApiResponse;
use crate::errors::{AppError, internal_error};

pub fn to_json<T: serde::Serialize>(
    result: Result<T>
) -> Result<Json<ApiResponse<T>>, AppError> {
    match result {
        Ok(value) => Ok(Json(ApiResponse::ok(value))),
        Err(err) => Err(internal_error(err)), // preserves original error string
    }
}
