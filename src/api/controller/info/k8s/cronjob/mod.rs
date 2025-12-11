use axum::extract::{Path, Query, State};
use axum::Json;
use k8s_openapi::api::batch::v1::CronJob;

use crate::api::dto::ApiResponse;
use crate::api::dto::info_dto::PaginationQuery;
use crate::api::dto::paginated_response::PaginatedResponse;
use crate::api::util::json::to_json;
use crate::app_state::AppState;
use crate::errors::AppError;

pub struct InfoK8sCronJobController;

impl InfoK8sCronJobController {
    pub async fn get_k8s_cronjobs(
        State(state): State<AppState>,
        Query(pagination): Query<PaginationQuery>,
    ) -> Result<Json<ApiResponse<PaginatedResponse<CronJob>>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .get_k8s_cronjobs_paginated(pagination.limit, pagination.offset)
                .await,
        )
    }

    pub async fn get_k8s_cronjob(
        Path((namespace, name)): Path<(String, String)>,
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<CronJob>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .get_k8s_cronjob(namespace, name)
                .await,
        )
    }
}
