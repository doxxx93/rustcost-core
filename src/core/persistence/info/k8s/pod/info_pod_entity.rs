use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents static and runtime information for a Kubernetes Pod.
///
/// Derived from Pod metadata (`.metadata`, `.spec`, `.status`) and runtime summary.
/// Stored at: `data/info/pod/{pod_uid}/info.rci`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InfoPodEntity {
    // --- Identity ---
    pub pod_name: Option<String>,
    pub namespace: Option<String>,
    pub pod_uid: Option<String>,

    // --- Lifecycle ---
    pub creation_timestamp: Option<DateTime<Utc>>,
    pub start_time: Option<DateTime<Utc>>,
    pub resource_version: Option<String>,

    pub last_updated_info_at: Option<DateTime<Utc>>,
    pub deleted: Option<bool>,
    pub last_check_deleted_count: Option<u64>,

    // --- Node association ---
    pub node_name: Option<String>,
    pub host_ip: Option<String>,
    pub pod_ip: Option<String>,

    // --- Status ---
    pub qos_class: Option<String>,
    pub phase: Option<String>,
    pub ready: Option<bool>,
    pub restart_count: Option<u32>,

    // --- Owner ---
    pub owner_kind: Option<String>,
    pub owner_name: Option<String>,
    pub owner_uid: Option<String>,

    // --- Containers ---
    pub container_count: Option<u32>,
    pub container_names: Option<Vec<String>>,
    pub container_images: Option<Vec<String>>,
    pub container_ids: Option<Vec<String>>,
    pub container_started_at: Option<Vec<DateTime<Utc>>>,
    pub image_ids: Option<Vec<String>>,
    pub container_ports: Option<Vec<u16>>,
    pub restart_policy: Option<String>,
    pub scheduler_name: Option<String>,
    pub service_account: Option<String>,

    // --- Volumes ---
    pub volume_count: Option<u32>,
    pub volume_names: Option<Vec<String>>,
    pub pvc_names: Option<Vec<String>>,
    pub mount_paths: Option<Vec<String>>,
    pub termination_grace_period_seconds: Option<u32>,
    pub tolerations: Option<Vec<String>>,

    // --- Metadata ---
    pub label: Option<String>,        // flattened "key=value,..."
    pub annotation: Option<String>,   // flattened "key=value,..."

    pub team: Option<String>,
    pub service: Option<String>,
    pub env: Option<String>, // "dev", "stage", "prod"
}

impl InfoPodEntity {
    /// Merge fields from `newer`, but preserve fields not returned by Kubernetes API.
    pub fn merge_from(&mut self, newer: InfoPodEntity) {
        // Only overwrite fields the API is responsible for:
        self.pod_name = newer.pod_name.or(self.pod_name.take());
        self.namespace = newer.namespace.or(self.namespace.take());
        self.pod_uid = newer.pod_uid.or(self.pod_uid.take());

        self.creation_timestamp = newer.creation_timestamp.or(self.creation_timestamp.take());
        self.start_time = newer.start_time.or(self.start_time.take());
        self.resource_version = newer.resource_version.or(self.resource_version.take());
        self.last_updated_info_at = newer.last_updated_info_at.or(self.last_updated_info_at.take());
        self.deleted = newer.deleted.or(self.deleted.take());
        self.last_check_deleted_count = newer.last_check_deleted_count.or(self.last_check_deleted_count.take());

        self.node_name = newer.node_name.or(self.node_name.take());
        self.host_ip = newer.host_ip.or(self.host_ip.take());
        self.pod_ip = newer.pod_ip.or(self.pod_ip.take());

        self.qos_class = newer.qos_class.or(self.qos_class.take());
        self.phase = newer.phase.or(self.phase.take());
        self.ready = newer.ready.or(self.ready.take());
        self.restart_count = newer.restart_count.or(self.restart_count.take());

        self.owner_kind = newer.owner_kind.or(self.owner_kind.take());
        self.owner_name = newer.owner_name.or(self.owner_name.take());
        self.owner_uid = newer.owner_uid.or(self.owner_uid.take());

        self.container_count = newer.container_count.or(self.container_count.take());
        self.container_names = newer.container_names.or(self.container_names.take());
        self.container_images = newer.container_images.or(self.container_images.take());
        self.container_ids = newer.container_ids.or(self.container_ids.take());
        self.image_ids = newer.image_ids.or(self.image_ids.take());
        self.container_ports = newer.container_ports.or(self.container_ports.take());
        self.restart_policy = newer.restart_policy.or(self.restart_policy.take());
        self.scheduler_name = newer.scheduler_name.or(self.scheduler_name.take());
        self.service_account = newer.service_account.or(self.service_account.take());

        self.volume_count = newer.volume_count.or(self.volume_count.take());
        self.volume_names = newer.volume_names.or(self.volume_names.take());
        self.pvc_names = newer.pvc_names.or(self.pvc_names.take());
        self.mount_paths = newer.mount_paths.or(self.mount_paths.take());
        self.termination_grace_period_seconds =
            newer.termination_grace_period_seconds.or(self.termination_grace_period_seconds.take());
        self.tolerations = newer.tolerations.or(self.tolerations.take());
        // DO NOT overwrite team/service/env – these are local annotations
        if newer.team.is_some() { self.team = newer.team; }
        if newer.service.is_some() { self.service = newer.service; }
        if newer.env.is_some() { self.env = newer.env; }
    }
}
