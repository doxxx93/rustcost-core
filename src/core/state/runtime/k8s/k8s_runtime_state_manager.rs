use std::sync::Arc;

use crate::core::state::runtime::k8s::k8s_runtime_state::{K8sRuntimeState, RuntimePod};
use crate::core::state::runtime::k8s::k8s_runtime_state_repository_trait::K8sRuntimeStateRepositoryTrait;

pub struct K8sRuntimeStateManager<R: K8sRuntimeStateRepositoryTrait> {
    pub(crate) repo: Arc<R>,
}

impl<R: K8sRuntimeStateRepositoryTrait> K8sRuntimeStateManager<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    /// Replace the entire K8s runtime state.
    pub async fn set_state(&self, state: K8sRuntimeState) {
        self.repo.set(state).await;
    }

    /// Update the discovery snapshot based on fresh K8s data.
    ///
    /// This expects a list of fully constructed RuntimePod entries.
    pub async fn update_discovery(
        &self,
        nodes: Vec<String>,
        namespaces: Vec<String>,
        deployments: Vec<String>,
        pods: Vec<RuntimePod>,
    ) {
        self.repo
            .update(|state| {
                state.update(nodes.clone(), namespaces.clone(), deployments.clone(), pods.clone());
            })
            .await;
    }

    /// Record a discovery failure (state remains intact).
    pub async fn mark_error(&self, message: String) {
        self.repo.update(|state| state.mark_error(message)).await;
    }
}
