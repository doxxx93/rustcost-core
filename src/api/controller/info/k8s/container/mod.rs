use axum::extract::{Path, Query, State};
use axum::Json;
use serde_json::Value;

use crate::api::util::json::to_json;
use crate::api::dto::info_dto::K8sListQuery;
use crate::api::dto::ApiResponse;
use crate::app_state::AppState;
use crate::core::persistence::info::k8s::container::info_container_entity::InfoContainerEntity;
use crate::domain::info::dto::info_k8s_container_patch_request::InfoK8sContainerPatchRequest;
use crate::errors::AppError;

pub struct InfoK8sContainerController;

impl InfoK8sContainerController {
    pub async fn get_info_k8s_container(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<Json<ApiResponse<InfoContainerEntity>>, AppError> {
        to_json(state.info_k8s_service.get_info_k8s_container(id).await)
    }

    pub async fn list_k8s_containers(
        State(state): State<AppState>,
        Query(filter): Query<K8sListQuery>,
    ) -> Result<Json<ApiResponse<Vec<InfoContainerEntity>>>, AppError> {
        to_json(state.info_k8s_service.list_k8s_containers(filter).await)
    }

    pub async fn patch_info_k8s_container(
        State(state): State<AppState>,
        Path(id): Path<String>,
        Json(payload): Json<InfoK8sContainerPatchRequest>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .patch_info_k8s_container(id, payload)
                .await,
        )
    }
}
