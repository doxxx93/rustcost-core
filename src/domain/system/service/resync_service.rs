use std::sync::Arc;

use anyhow::Result;
use serde_json::{json, Value};
use tracing::error;
use crate::core::state::runtime::k8s::k8s_runtime_state_manager::K8sRuntimeStateManager;
use crate::core::state::runtime::k8s::k8s_runtime_state_repository::K8sRuntimeStateRepository;
use crate::scheduler::tasks::info::k8s_refresh::task::refresh_k8s_object_info;

pub async fn resync(
    k8s_state: Arc<K8sRuntimeStateManager<K8sRuntimeStateRepository>>,
) -> Result<Value> {
    do_resync(k8s_state).await
}

/// Kick off a background refresh of the Kubernetes runtime state.
pub async fn do_resync(
    k8s_state: Arc<K8sRuntimeStateManager<K8sRuntimeStateRepository>>,
) -> Result<Value> {
    tokio::spawn(async move {
        if let Err(e) = refresh_k8s_object_info(&k8s_state).await {
            error!("K8s resync failed: {e}");
        }
    });

    Ok(json!({ "resync": "started" }))
}
