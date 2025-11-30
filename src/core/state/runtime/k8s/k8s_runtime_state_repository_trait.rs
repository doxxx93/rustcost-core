use std::sync::Arc;
use async_trait::async_trait;

use crate::core::state::runtime::k8s::k8s_runtime_state::K8sRuntimeState;

#[async_trait]
pub trait K8sRuntimeStateRepositoryTrait: Send + Sync {

    /// Return the current state as an Arc.
    /// This avoids cloning the entire state (~2–3 MB with 10k pods).
    async fn get(&self) -> Arc<K8sRuntimeState>;

    /// Replace the entire state.
    async fn set(&self, state: K8sRuntimeState);

    /// Mutate the internal state using a closure.
    async fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut K8sRuntimeState) + Send + Sync;
}
