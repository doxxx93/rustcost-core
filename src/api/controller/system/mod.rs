//! System controller: connects routes to system usecases

use axum::extract::{Path, Query};
use axum::Json;
use serde_json::Value;

use crate::api::dto::ApiResponse;
use crate::api::dto::system_dto::{LogQuery, PaginatedLogResponse};
use crate::core::persistence::logs::log_repository::{LogRepository, LogRepositoryImpl};
use crate::errors::AppError;

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
pub async fn get_system_log_file_list() -> Json<ApiResponse<Value>> {
    match crate::domain::system::service::log_service::get_system_log_file_list().await {
        Ok(v) => Json(ApiResponse::ok(v)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}


pub async fn get_system_log_lines(
    Path(date): Path<String>,
    Query(query): Query<LogQuery>,
) -> Result<Json<ApiResponse<PaginatedLogResponse>>, AppError> {
    let cursor = query.cursor.unwrap_or(0);
    let limit = query.limit.unwrap_or(100);

    let repo = LogRepositoryImpl::new();

    let (lines, next_cursor) = repo
        .get_system_log_lines(&date, cursor, limit)
        .await
        .map_err(|_e| AppError::InternalServerError)?;

    let response = ApiResponse::ok(PaginatedLogResponse {
        date,
        lines,
        next_cursor,
    });

    Ok(Json(response))
}

