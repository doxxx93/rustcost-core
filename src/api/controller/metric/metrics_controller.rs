//! Metrics controller helpers: common response mapping utilities.

use anyhow::Result;
use axum::Json;

use crate::api::dto::ApiResponse;

/// Map a domain Result<Value> into Json<ApiResponse<Value>>
pub fn to_json<T: serde::Serialize>(result: Result<T>) -> Json<ApiResponse<T>> {
    match result {
        Ok(v) => Json(ApiResponse::ok(v)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}