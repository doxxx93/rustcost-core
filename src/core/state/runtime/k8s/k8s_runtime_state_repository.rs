use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::state::runtime::k8s::k8s_runtime_state::K8sRuntimeState;
use crate::core::state::runtime::k8s::k8s_runtime_state_repository_trait::K8sRuntimeStateRepositoryTrait;

pub struct K8sRuntimeStateRepository {
    state: Arc<RwLock<Arc<K8sRuntimeState>>>,
}

impl K8sRuntimeStateRepository {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(Arc::new(K8sRuntimeState::default()))),
        }
    }

    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}

#[async_trait::async_trait]
impl K8sRuntimeStateRepositoryTrait for K8sRuntimeStateRepository {
    /// Return the shared Arc snapshot (zero cost).
    async fn get(&self) -> Arc<K8sRuntimeState> {
        self.state.read().await.clone()
    }

    /// Replace entire state atomically.
    async fn set(&self, new_state: K8sRuntimeState) {
        let mut guard = self.state.write().await;
        *guard = Arc::new(new_state);
    }

    /// Mutate the internal state by cloning and updating.
    async fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut K8sRuntimeState) + Send + Sync,
    {
        let mut guard = self.state.write().await;

        // Clone underlying state
        let mut new_state = (**guard).clone();

        // Apply mutation
        f(&mut new_state);

        // Replace Arc pointer
        *guard = Arc::new(new_state);
    }
}
