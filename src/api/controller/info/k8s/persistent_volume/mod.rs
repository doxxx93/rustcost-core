use axum::extract::{Path, Query, State};
use axum::Json;
use k8s_openapi::api::core::v1::PersistentVolume;

use crate::api::dto::ApiResponse;
use crate::api::dto::info_dto::PaginationQuery;
use crate::api::dto::paginated_response::PaginatedResponse;
use crate::api::util::json::to_json;
use crate::app_state::AppState;
use crate::errors::AppError;

pub struct InfoK8sPersistentVolumeController;

impl InfoK8sPersistentVolumeController {
    pub async fn get_k8s_persistent_volumes(
        State(state): State<AppState>,
        Query(pagination): Query<PaginationQuery>,
    ) -> Result<Json<ApiResponse<PaginatedResponse<PersistentVolume>>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .get_k8s_persistent_volumes_paginated(pagination.limit, pagination.offset)
                .await,
        )
    }

    pub async fn get_k8s_persistent_volume(
        Path(name): Path<String>,
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<PersistentVolume>>, AppError> {
        to_json(state.info_k8s_service.get_k8s_persistent_volume(name).await)
    }
}
