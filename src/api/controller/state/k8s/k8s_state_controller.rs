use axum::{extract::State, Json};
use serde_json::{json, Value};
use crate::api::dto::ApiResponse;
use crate::api::util::json::to_json;
use crate::app_state::AppState;
use crate::core::state::runtime::k8s::k8s_runtime_state_repository_trait::K8sRuntimeStateRepositoryTrait;
use crate::errors::AppError;

pub struct K8sStateController;

impl K8sStateController {
    pub async fn get_full(
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        let s = state.k8s_state.repo.get().await;   // Arc<K8sRuntimeState>
        to_json(Ok(json!(&*s)))              // FIX: serialize the inner value
    }

    pub async fn get_summary(
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        let s = state.k8s_state.repo.get().await;

        // calculate container count
        let container_count: usize = s
            .pods
            .values()
            .map(|p| p.containers.len())
            .sum();

        to_json(Ok(json!({
            "nodes": s.nodes.len(),
            "namespaces": s.namespaces.len(),
            "deployments": s.deployments.len(),
            "pods": s.pods.len(),
            "containers": container_count,
            "last_discovered_at": s.last_discovered_at,
            "last_error_at": s.last_error_at,
            "last_error_message": s.last_error_message,
        })))
    }
}
