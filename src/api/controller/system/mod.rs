//! System controller: connects routes to system usecases

use axum::extract::{Path, Query, State};
use axum::Json;
use serde_json::Value;


use crate::api::dto::system_dto::{LogQuery, PaginatedLogResponse};
use crate::api::dto::ApiResponse;
use crate::api::util::json::to_json;
use crate::app_state::AppState;
use crate::errors::AppError;

pub struct SystemController;

impl SystemController {
    pub async fn status(
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.system_service.status().await)
    }

    pub async fn health(
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.system_service.health().await)
    }

    pub async fn backup(
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.system_service.backup().await)
    }

    pub async fn resync(
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.system_service.resync().await)
    }

    pub async fn get_system_log_file_list(
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
        to_json(state.log_service.get_system_log_file_list().await)
    }

    pub async fn get_system_log_lines(
        State(state): State<AppState>,
        Path(date): Path<String>,
        Query(query): Query<LogQuery>,
    ) -> Result<Json<ApiResponse<PaginatedLogResponse>>, AppError> {
        to_json(
            state
                .log_service
                .get_system_log_lines(&date, query.cursor, query.limit)
                .await,
        )
    }
}

