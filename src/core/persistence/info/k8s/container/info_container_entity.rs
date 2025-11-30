use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents static and runtime information for a Kubernetes **Container**.
///
/// Derived from Pod/Container metadata and Kubelet `/stats/summary`.
/// Stored at: `data/info/container/{pod_uid}-{container_name}/info.rci`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InfoContainerEntity {
    // --- Identity ---
    /// UID of the parent Pod
    pub pod_uid: Option<String>,
    /// Name of the parent Pod
    pub pod_name: Option<String>,
    /// Container name (unique within the Pod)
    pub container_name: Option<String>,
    /// Namespace of the Pod
    pub namespace: Option<String>,

    // --- Lifecycle ---
    /// When the container spec was first created
    pub creation_timestamp: Option<DateTime<Utc>>,
    /// When the container actually started
    pub start_time: Option<DateTime<Utc>>,
    /// Container runtime ID (e.g. "docker://...", "containerd://...")
    pub container_id: Option<String>,
    /// Image name used
    pub image: Option<String>,
    /// Image ID hash (from runtime)
    pub image_id: Option<String>,

    // --- Status ---
    /// Current container state: "Running", "Waiting", "Terminated"
    pub state: Option<String>,
    /// Reason if Waiting/Terminated
    pub reason: Option<String>,
    /// Message if Waiting/Terminated
    pub message: Option<String>,
    /// Exit code if Terminated
    pub exit_code: Option<i32>,
    /// Last restart count
    pub restart_count: Option<i32>,
    /// Whether container is currently ready
    pub ready: Option<bool>,

    // --- Node association ---
    pub node_name: Option<String>,
    pub host_ip: Option<String>,
    pub pod_ip: Option<String>,

    // --- Resources ---
    /// Requested CPU (cores)
    pub cpu_request_millicores: Option<u64>,
    /// Requested Memory (bytes)
    pub memory_request_bytes: Option<u64>,
    /// CPU limit (cores)
    pub cpu_limit_millicores: Option<u64>,
    /// Memory limit (bytes)
    pub memory_limit_bytes: Option<u64>,

    // --- Volumes and mounts ---
    pub volume_mounts: Option<Vec<String>>,
    pub volume_devices: Option<Vec<String>>,

    // --- Metadata ---
    pub labels: Option<String>,       // "key=value,..."
    pub annotations: Option<String>,  // "key=value,..."

    // --- Bookkeeping ---
    pub last_updated_info_at: Option<DateTime<Utc>>,
    pub deleted: Option<bool>,
    pub last_check_deleted_count: Option<u64>,

    // --- Team / Service metadata (NEW) ---
    pub team: Option<String>,
    pub service: Option<String>,
    pub env: Option<String>, // "dev", "stage", "prod"
}

impl InfoContainerEntity {
    /// Merge container data coming from API, preserving any user-set metadata.
    pub fn merge_from(&mut self, newer: InfoContainerEntity) {
        self.pod_uid = newer.pod_uid.or(self.pod_uid.take());
        self.pod_name = newer.pod_name.or(self.pod_name.take());
        self.container_name = newer.container_name.or(self.container_name.take());
        self.namespace = newer.namespace.or(self.namespace.take());

        self.creation_timestamp = newer.creation_timestamp.or(self.creation_timestamp.take());
        self.start_time = newer.start_time.or(self.start_time.take());
        self.container_id = newer.container_id.or(self.container_id.take());
        self.image = newer.image.or(self.image.take());
        self.image_id = newer.image_id.or(self.image_id.take());

        self.state = newer.state.or(self.state.take());
        self.reason = newer.reason.or(self.reason.take());
        self.message = newer.message.or(self.message.take());
        self.exit_code = newer.exit_code.or(self.exit_code.take());
        self.restart_count = newer.restart_count.or(self.restart_count.take());
        self.ready = newer.ready.or(self.ready.take());

        self.node_name = newer.node_name.or(self.node_name.take());
        self.host_ip = newer.host_ip.or(self.host_ip.take());
        self.pod_ip = newer.pod_ip.or(self.pod_ip.take());

        self.cpu_request_millicores =
            newer.cpu_request_millicores.or(self.cpu_request_millicores.take());
        self.memory_request_bytes =
            newer.memory_request_bytes.or(self.memory_request_bytes.take());
        self.cpu_limit_millicores =
            newer.cpu_limit_millicores.or(self.cpu_limit_millicores.take());
        self.memory_limit_bytes =
            newer.memory_limit_bytes.or(self.memory_limit_bytes.take());

        self.volume_mounts = newer.volume_mounts.or(self.volume_mounts.take());
        self.volume_devices = newer.volume_devices.or(self.volume_devices.take());

        self.labels = newer.labels.or(self.labels.take());
        self.annotations = newer.annotations.or(self.annotations.take());

        self.last_updated_info_at =
            newer.last_updated_info_at.or(self.last_updated_info_at.take());
        self.deleted = newer.deleted.or(self.deleted.take());
        self.last_check_deleted_count =
            newer.last_check_deleted_count.or(self.last_check_deleted_count.take());

        // Preserve team/service/env
        if newer.team.is_some()     { self.team = newer.team; }
        if newer.service.is_some()  { self.service = newer.service; }
        if newer.env.is_some()      { self.env = newer.env; }
    }
}