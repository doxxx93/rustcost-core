use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents static and runtime information for a Kubernetes node.
///
/// Combines metadata (from the Node resource) and metrics (from metrics-server or API).
/// Stored at `data/info/node/{node_name}/info.rci`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InfoNodeEntity {
    // --- Identity & Metadata ---
    pub node_name: Option<String>,
    pub node_uid: Option<String>,
    pub creation_timestamp: Option<DateTime<Utc>>,
    pub resource_version: Option<String>,

    // --- Lifecycle ---
    pub last_updated_info_at: Option<DateTime<Utc>>,
    pub deleted: Option<bool>,
    pub last_check_deleted_count: Option<u64>,

    // --- Host Info ---
    pub hostname: Option<String>,
    pub internal_ip: Option<String>,
    pub architecture: Option<String>,
    pub os_image: Option<String>,
    pub kernel_version: Option<String>,
    pub kubelet_version: Option<String>,
    pub container_runtime: Option<String>,
    pub operating_system: Option<String>,

    // --- Capacity ---
    pub cpu_capacity_cores: Option<u32>,
    pub memory_capacity_bytes: Option<u64>,
    pub pod_capacity: Option<u32>,
    pub ephemeral_storage_capacity_bytes: Option<u64>,

    // --- Allocatable ---
    pub cpu_allocatable_cores: Option<u32>,
    pub memory_allocatable_bytes: Option<u64>,
    pub ephemeral_storage_allocatable_bytes: Option<u64>,
    pub pod_allocatable: Option<u32>,

    // --- Status ---
    pub ready: Option<bool>,
    pub taints: Option<String>,
    pub label: Option<String>,
    pub annotation: Option<String>,

    // --- Images ---
    pub image_count: Option<u32>,
    pub image_names: Option<Vec<String>>,
    pub image_total_size_bytes: Option<u64>,

    // --- Cost (node-specific) ---
    /// Fixed price for this node in USD (instance / VM / bare metal)
    pub fixed_instance_usd: Option<f64>,

    /// Billing period for `fixed_instance`
    pub price_period: Option<NodePricePeriod>,

    pub team: Option<String>,
    pub service: Option<String>,
    pub env: Option<String>, // "dev", "stage", "prod"

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodePricePeriod {
    /// Unit-based pricing (CPU-hour, GB-hour, etc.)
    Unit,

    /// Fixed price per hour
    Hour,

    /// Fixed price per day
    Day,

    /// Fixed price per month
    Month,
}

impl InfoNodeEntity {
    /// Merge data from API (`newer`), preserving user-managed fields.
    pub fn merge_from(&mut self, newer: InfoNodeEntity) {
        self.node_name = newer.node_name.or(self.node_name.take());
        self.node_uid = newer.node_uid.or(self.node_uid.take());
        self.creation_timestamp = newer.creation_timestamp.or(self.creation_timestamp.take());
        self.resource_version = newer.resource_version.or(self.resource_version.take());

        self.last_updated_info_at =
            newer.last_updated_info_at.or(self.last_updated_info_at.take());
        self.deleted = newer.deleted.or(self.deleted.take());
        self.last_check_deleted_count =
            newer.last_check_deleted_count.or(self.last_check_deleted_count.take());

        self.hostname = newer.hostname.or(self.hostname.take());
        self.internal_ip = newer.internal_ip.or(self.internal_ip.take());
        self.architecture = newer.architecture.or(self.architecture.take());
        self.os_image = newer.os_image.or(self.os_image.take());
        self.kernel_version = newer.kernel_version.or(self.kernel_version.take());
        self.kubelet_version = newer.kubelet_version.or(self.kubelet_version.take());
        self.container_runtime = newer.container_runtime.or(self.container_runtime.take());
        self.operating_system = newer.operating_system.or(self.operating_system.take());

        self.cpu_capacity_cores = newer.cpu_capacity_cores.or(self.cpu_capacity_cores.take());
        self.memory_capacity_bytes =
            newer.memory_capacity_bytes.or(self.memory_capacity_bytes.take());
        self.pod_capacity = newer.pod_capacity.or(self.pod_capacity.take());
        self.ephemeral_storage_capacity_bytes = newer
            .ephemeral_storage_capacity_bytes
            .or(self.ephemeral_storage_capacity_bytes.take());

        self.cpu_allocatable_cores =
            newer.cpu_allocatable_cores.or(self.cpu_allocatable_cores.take());
        self.memory_allocatable_bytes =
            newer.memory_allocatable_bytes.or(self.memory_allocatable_bytes.take());
        self.ephemeral_storage_allocatable_bytes = newer
            .ephemeral_storage_allocatable_bytes
            .or(self.ephemeral_storage_allocatable_bytes.take());
        self.pod_allocatable = newer.pod_allocatable.or(self.pod_allocatable.take());

        self.ready = newer.ready.or(self.ready.take());
        self.taints = newer.taints.or(self.taints.take());
        self.label = newer.label.or(self.label.take());
        self.annotation = newer.annotation.or(self.annotation.take());

        self.image_count = newer.image_count.or(self.image_count.take());
        self.image_names = newer.image_names.or(self.image_names.take());
        self.image_total_size_bytes =
            newer.image_total_size_bytes.or(self.image_total_size_bytes.take());

        // Preserve user-provided metadata (local annotations)
        if newer.team.is_some() { self.team = newer.team; }
        if newer.service.is_some() { self.service = newer.service; }
        if newer.env.is_some() { self.env = newer.env; }

        // Preserve user-managed node pricing
        if newer.fixed_instance_usd.is_some() {
            self.fixed_instance_usd = newer.fixed_instance_usd;
        }
        if newer.price_period.is_some() {
            self.price_period = newer.price_period;
        }
    }
}