use axum::extract::{Path, Query, State};
use axum::Json;
use serde_json::Value;

use crate::api::util::json::to_json;
use crate::api::dto::info_dto::K8sListQuery;
use crate::api::dto::ApiResponse;
use crate::app_state::AppState;
use crate::core::persistence::info::k8s::pod::info_pod_entity::InfoPodEntity;
use crate::domain::info::dto::info_k8s_pod_patch_request::InfoK8sPodPatchRequest;
use crate::errors::AppError;

pub struct InfoK8sPodController;

impl InfoK8sPodController {
    pub async fn get_info_k8s_pod(
        State(state): State<AppState>,
        Path(pod_uid): Path<String>,
    ) -> Result<Json<ApiResponse<InfoPodEntity>>, AppError> {
        to_json(state.info_k8s_service.get_info_k8s_pod(pod_uid).await)
    }

    /// List pods – optionally filter by `namespace`, `labelSelector`, or `nodeName`
    pub async fn list_k8s_pods(
        State(state): State<AppState>,
        Query(filter): Query<K8sListQuery>,
    ) -> Result<Json<ApiResponse<Vec<InfoPodEntity>>>, AppError> {
        to_json(state.info_k8s_service.list_k8s_pods(filter).await)
    }

    pub async fn patch_info_k8s_pod(
        State(state): State<AppState>,
        Path(id): Path<String>,
        Json(payload): Json<InfoK8sPodPatchRequest>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.info_k8s_service.patch_info_k8s_pod(id, payload).await)
    }
}
