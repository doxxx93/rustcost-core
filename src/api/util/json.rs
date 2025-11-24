use anyhow::Result;
use axum::Json;

use crate::api::dto::ApiResponse;
use crate::errors::AppError;

pub fn to_json<T: serde::Serialize>(
    result: Result<T>
) -> Result<Json<ApiResponse<T>>, AppError> {
    match result {
        Ok(v) => Ok(Json(ApiResponse::ok(v))),
        Err(_) => Err(AppError::InternalServerError),
    }
}
