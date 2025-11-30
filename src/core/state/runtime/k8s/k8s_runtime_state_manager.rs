use std::sync::Arc;
use chrono::Utc;
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
    ) -> anyhow::Result<()> {
        self.repo
            .update(|state| {
                state.update(nodes.clone(), namespaces.clone(), deployments.clone(), pods.clone());
            })
            .await;

        Ok(())
    }

    /// Record a discovery failure (state remains intact).
    pub async fn mark_error(&self, message: String) {
        self.repo.update(|state| state.mark_error(message)).await;
    }

    // ===============================================
    // 1. Is last discovery recent (< 3 hours)
    // ===============================================
    pub async fn is_fresh(&self) -> bool {
        let state = self.repo.get().await;

        if let Some(ts) = state.last_discovered_at {
            let hours = (Utc::now() - ts).num_hours();
            return hours < 3;
        }
        false
    }

    // ===============================================
    // 2. Get pods by deployment (fast O(n))
    // ===============================================
    pub async fn get_pods_by_deployment(&self, deployment: &str) -> Vec<RuntimePod> {
        let state = self.repo.get().await;

        if let Some(uids) = state.pods_by_deployment.get(deployment) {
            return uids
                .iter()
                .filter_map(|uid| state.pods.get(uid).cloned())
                .collect();
        }
        Vec::new()
    }

    // ===============================================
    // 3. Get pods by namespace
    // ===============================================
    pub async fn get_pods_by_namespace(&self, namespace: &str) -> Vec<RuntimePod> {
        let state = self.repo.get().await;

        if let Some(uids) = state.pods_by_namespace.get(namespace) {
            return uids
                .iter()
                .filter_map(|uid| state.pods.get(uid).cloned())
                .collect();
        }
        Vec::new()
    }

    // ===============================================
    // 4. Get pods by node
    // ===============================================
    pub async fn get_pods_by_node(&self, node: &str) -> Vec<RuntimePod> {
        let state = self.repo.get().await;

        if let Some(uids) = state.pods_by_node.get(node) {
            return uids
                .iter()
                .filter_map(|uid| state.pods.get(uid).cloned())
                .collect();
        }
        Vec::new()
    }

    // ===============================================
    // 5. Get all node names
    // ===============================================
    pub async fn get_nodes(&self) -> Vec<String> {
        let state = self.repo.get().await;
        state.nodes.clone()
    }

    // ===============================================
    // 6. Get all namespaces
    // ===============================================
    pub async fn get_namespaces(&self) -> Vec<String> {
        let state = self.repo.get().await;
        state.namespaces.clone()
    }

    // ===============================================
    // 7. Get all containers for a pod UID
    // ===============================================
    pub async fn get_containers(&self, pod_uid: &str) -> Vec<String> {
        let state = self.repo.get().await;
        state
            .pods
            .get(pod_uid)
            .map(|p| p.containers.clone())
            .unwrap_or_default()
    }
}
