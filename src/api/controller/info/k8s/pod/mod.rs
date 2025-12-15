use axum::extract::{Path, Query, State};
use axum::Json;
use serde_json::Value;

use crate::api::util::json::to_json;
use crate::api::dto::ApiResponse;
use crate::api::dto::info_dto::PaginationQuery;
use crate::api::dto::k8s_pod_query_request_dto::K8sPodQueryRequestDto;
use crate::api::dto::paginated_response::PaginatedResponse;
use crate::app_state::AppState;
use crate::core::persistence::info::k8s::pod::info_pod_entity::InfoPodEntity;
use crate::domain::info::dto::info_k8s_pod_patch_request::InfoK8sPodPatchRequest;
use crate::errors::AppError;
use k8s_openapi::api::core::v1::Pod;

pub struct InfoK8sPodController;
pub struct InfoK8sLivePodController;

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
        Query(filter): Query<K8sPodQueryRequestDto>,
    ) -> Result<Json<ApiResponse<PaginatedResponse<InfoPodEntity>>>, AppError> {
        let svc = state.info_k8s_service.clone();
        let state_clone = state.clone();
        to_json(svc.list_k8s_pods(state_clone, filter).await)
    }

    pub async fn patch_info_k8s_pod(
        State(state): State<AppState>,
        Path(id): Path<String>,
        Json(payload): Json<InfoK8sPodPatchRequest>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.info_k8s_service.patch_info_k8s_pod(id, payload).await)
    }
}

impl InfoK8sLivePodController {
    pub async fn list_k8s_pods(
        State(state): State<AppState>,
        Query(pagination): Query<PaginationQuery>,
    ) -> Result<Json<ApiResponse<PaginatedResponse<Pod>>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .get_k8s_live_pods_paginated(pagination.limit, pagination.offset)
                .await,
        )
    }

    pub async fn get_k8s_pod(
        Path(pod_uid): Path<String>,
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<Pod>>, AppError> {
        to_json(state.info_k8s_service.get_k8s_live_pod(pod_uid).await)
    }
}
