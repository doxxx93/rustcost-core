use axum::extract::{Path, Query, State};
use axum::Json;
use k8s_openapi::api::batch::v1::Job;

use crate::api::dto::ApiResponse;
use crate::api::dto::info_dto::PaginationQuery;
use crate::api::dto::paginated_response::PaginatedResponse;
use crate::api::util::json::to_json;
use crate::app_state::AppState;
use crate::errors::AppError;

pub struct InfoK8sJobController;

impl InfoK8sJobController {
    pub async fn get_k8s_jobs(
        State(state): State<AppState>,
        Query(pagination): Query<PaginationQuery>,
    ) -> Result<Json<ApiResponse<PaginatedResponse<Job>>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .get_k8s_jobs_paginated(pagination.limit, pagination.offset)
                .await,
        )
    }

    pub async fn get_k8s_job(
        Path((namespace, name)): Path<(String, String)>,
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<Job>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .get_k8s_job(namespace, name)
                .await,
        )
    }
}
