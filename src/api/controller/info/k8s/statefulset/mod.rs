use axum::extract::{Path, Query, State};
use axum::Json;
use k8s_openapi::api::apps::v1::StatefulSet;

use crate::api::dto::ApiResponse;
use crate::api::dto::info_dto::PaginationQuery;
use crate::api::dto::paginated_response::PaginatedResponse;
use crate::api::util::json::to_json;
use crate::app_state::AppState;
use crate::errors::AppError;

pub struct InfoK8sStatefulSetController;

impl InfoK8sStatefulSetController {
    pub async fn get_k8s_statefulsets(
        State(state): State<AppState>,
        Query(pagination): Query<PaginationQuery>,
    ) -> Result<Json<ApiResponse<PaginatedResponse<StatefulSet>>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .get_k8s_statefulsets_paginated(pagination.limit, pagination.offset)
                .await,
        )
    }

    pub async fn get_k8s_statefulset(
        Path((namespace, name)): Path<(String, String)>,
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<StatefulSet>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .get_k8s_statefulset(namespace, name)
                .await,
        )
    }
}
