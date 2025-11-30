use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a single Kubernetes pod with relationships.
/// This structure is optimized for in-memory use and fast lookups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimePod {
    /// Immutable Kubernetes identifier
    pub uid: String,

    /// Human-readable pod name (may change when recreated)
    pub name: String,

    /// Namespace the pod belongs to
    pub namespace: String,

    /// Top-level deployment (if applicable)
    pub deployment: Option<String>,

    /// Node where the pod is running
    pub node: String,

    /// Container names under this pod
    pub containers: Vec<String>,
}

/// In-memory runtime snapshot of all Kubernetes objects discovered by RustCost.
///
/// This state:
/// - lives only in memory (NOT persisted)
/// - is overwritten each discovery cycle
/// - is designed to handle 10k+ pods efficiently
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sRuntimeState {
    // ===== Timestamps =====
    pub last_discovered_at: Option<DateTime<Utc>>,
    pub last_error_at: Option<DateTime<Utc>>,

    // ===== Basic sets (names only) =====
    pub nodes: Vec<String>,
    pub namespaces: Vec<String>,
    pub deployments: Vec<String>,

    // ===== Detailed objects =====
    /// Pods keyed by UID for O(1) lookup
    pub pods: HashMap<String, RuntimePod>,

    // ===== Secondary indexes (for fast filtering) =====
    /// namespace → pod_uids
    pub pods_by_namespace: HashMap<String, Vec<String>>,
    /// node → pod_uids
    pub pods_by_node: HashMap<String, Vec<String>>,
    /// deployment → pod_uids
    pub pods_by_deployment: HashMap<String, Vec<String>>,

    // ===== Optional: last discovery error =====
    pub last_error_message: Option<String>,
}

impl Default for K8sRuntimeState {
    fn default() -> Self {
        Self {
            last_discovered_at: None,
            last_error_at: None,

            nodes: Vec::new(),
            namespaces: Vec::new(),
            deployments: Vec::new(),

            pods: HashMap::new(),
            pods_by_namespace: HashMap::new(),
            pods_by_node: HashMap::new(),
            pods_by_deployment: HashMap::new(),

            last_error_message: None,
        }
    }
}

impl K8sRuntimeState {
    /// Fully replace the runtime state with new values from a discovery cycle.
    ///
    /// This is optimized for 10k+ pods and avoids unnecessary allocations.
    pub fn update(
        &mut self,
        nodes: Vec<String>,
        namespaces: Vec<String>,
        deployments: Vec<String>,
        pods: Vec<RuntimePod>,
    ) {
        // Basic items
        self.nodes = nodes;
        self.namespaces = namespaces;
        self.deployments = deployments;

        // Clear old lookup tables
        self.pods.clear();
        self.pods_by_namespace.clear();
        self.pods_by_node.clear();
        self.pods_by_deployment.clear();

        // Fill HashMaps + indexes
        for pod in pods {
            let uid = pod.uid.clone();

            // Insert pod
            self.pods.insert(uid.clone(), pod.clone());

            // Index by namespace
            self.pods_by_namespace
                .entry(pod.namespace.clone())
                .or_default()
                .push(uid.clone());

            // Index by node
            self.pods_by_node
                .entry(pod.node.clone())
                .or_default()
                .push(uid.clone());

            // Index by deployment
            if let Some(depl) = &pod.deployment {
                self.pods_by_deployment
                    .entry(depl.clone())
                    .or_default()
                    .push(uid.clone());
            }
        }

        // Discovery timestamp
        self.last_discovered_at = Some(Utc::now());
        self.last_error_at = None;
        self.last_error_message = None;
    }

    /// Mark an error during discovery without modifying the object lists.
    pub fn mark_error(&mut self, msg: String) {
        self.last_error_message = Some(msg);
        self.last_error_at = Some(Utc::now());
    }

    /// Reset everything (rarely needed)
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
