//! System controller: connects routes to system usecases

use axum::extract::Path;
use axum::Json;
use serde_json::Value;

use crate::api::dto::ApiResponse;

pub async fn status() -> Json<ApiResponse<Value>> {
    match crate::domain::system::service::status_service::status().await {
        Ok(v) => Json(ApiResponse::ok(v)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}

pub async fn health() -> Json<ApiResponse<Value>> {
    match crate::domain::system::service::health_service::health().await {
        Ok(v) => Json(ApiResponse::ok(v)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}

pub async fn backup() -> Json<ApiResponse<Value>> {
    match crate::domain::system::service::backup_service::backup().await {
        Ok(v) => Json(ApiResponse::ok(v)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}

pub async fn resync() -> Json<ApiResponse<Value>> {
    match crate::domain::system::service::resync_service::resync().await {
        Ok(v) => Json(ApiResponse::ok(v)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}

pub async fn list_logs() -> Json<ApiResponse<Value>> {
    match crate::domain::system::service::log_service::list_logs().await {
        Ok(v) => Json(ApiResponse::ok(v)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}

pub async fn show_log_by_date(Path(date): Path<String>,) -> Json<ApiResponse<Value>> {
    match crate::domain::system::service::log_service::show_log_by_date(date).await {
        Ok(v) => Json(ApiResponse::ok(v)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}
