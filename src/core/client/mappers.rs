/// Maps kube-rs / k8s-openapi types → internal domain models
use crate::core::client::kube_resources::{Node, Pod, Deployment, Namespace, HorizontalPodAutoscaler,
    PersistentVolume, PersistentVolumeClaim, ResourceQuota, LimitRange};
use crate::core::persistence::info::k8s::node::info_node_entity::InfoNodeEntity;
use crate::core::persistence::info::k8s::pod::info_pod_entity::InfoPodEntity;
use crate::core::persistence::info::k8s::deployment::info_deployment_entity::InfoDeploymentEntity;
use crate::core::persistence::info::k8s::namespace::info_namespace_entity::InfoNamespaceEntity;
use anyhow::Result;
use chrono::{DateTime, Utc};

/// Converts a k8s-openapi Node object into an InfoNodeEntity
pub fn map_node_to_info_entity(node: &Node) -> Result<InfoNodeEntity> {
    let metadata = &node.metadata;
    let status = node.status.as_ref();
    let spec = node.spec.as_ref();

    // Parse creation timestamp
    let creation_timestamp = metadata
        .creation_timestamp
        .as_ref()
        .map(|ts| DateTime::parse_from_rfc3339(&ts.0.to_rfc3339()))
        .and_then(|r| r.ok())
        .map(|dt| dt.with_timezone(&Utc));

    let last_updated_info_at = Some(Utc::now());

    // Extract addresses (hostname, internal IP)
    let (hostname, internal_ip) = status
        .and_then(|s| s.addresses.as_ref())
        .map(|addresses| {
            let mut hostname = None;
            let mut internal_ip = None;
            for addr in addresses {
                match addr.type_.as_str() {
                    "Hostname" => hostname = Some(addr.address.clone()),
                    "InternalIP" => internal_ip = Some(addr.address.clone()),
                    _ => {}
                }
            }
            (hostname, internal_ip)
        })
        .unwrap_or_default();

    // Extract NodeSystemInfo
    let sys_info = status.and_then(|s| s.node_info.as_ref());
    let (architecture, os_image, kernel_version, kubelet_version, container_runtime, operating_system) =
        sys_info
            .map(|info| {
                (
                    Some(info.architecture.clone()),
                    Some(info.os_image.clone()),
                    Some(info.kernel_version.clone()),
                    Some(info.kubelet_version.clone()),
                    Some(info.container_runtime_version.clone()),
                    Some(info.operating_system.clone()),
                )
            })
            .unwrap_or_default();

    // Parse capacities and allocatables
    let capacity = status.and_then(|s| s.capacity.as_ref());
    let allocatable = status.and_then(|s| s.allocatable.as_ref());

    let parse_cpu = |map: Option<&std::collections::BTreeMap<String, k8s_openapi::apimachinery::pkg::api::resource::Quantity>>| {
        map.and_then(|m| m.get("cpu"))
            .and_then(|q| {
                let s = &q.0;
                if s.ends_with('m') {
                    s.trim_end_matches('m').parse::<u32>().ok().map(|millicores| millicores / 1000)
                } else {
                    s.parse::<u32>().ok()
                }
            })
    };

    let parse_mem = |map: Option<&std::collections::BTreeMap<String, k8s_openapi::apimachinery::pkg::api::resource::Quantity>>| {
        map.and_then(|m| m.get("memory"))
            .and_then(|q| {
                let s = q.0.to_lowercase();
                if s.ends_with("ki") {
                    s.trim_end_matches("ki").parse::<u64>().ok().map(|v| v * 1024)
                } else if s.ends_with('k') {
                    s.trim_end_matches('k').parse::<u64>().ok().map(|v| v * 1000)
                } else if s.ends_with("mi") {
                    s.trim_end_matches("mi").parse::<u64>().ok().map(|v| v * 1024 * 1024)
                } else if s.ends_with('m') {
                    s.trim_end_matches('m').parse::<u64>().ok().map(|v| v * 1000 * 1000)
                } else if s.ends_with("gi") {
                    s.trim_end_matches("gi").parse::<u64>().ok().map(|v| v * 1024 * 1024 * 1024)
                } else if s.ends_with('g') {
                    s.trim_end_matches('g').parse::<u64>().ok().map(|v| v * 1000 * 1000 * 1000)
                } else {
                    s.parse::<u64>().ok()
                }
            })
    };

    let parse_storage = |map: Option<&std::collections::BTreeMap<String, k8s_openapi::apimachinery::pkg::api::resource::Quantity>>| {
        map.and_then(|m| m.get("ephemeral-storage"))
            .and_then(|q| {
                let s = q.0.to_lowercase();
                if s.ends_with("ki") {
                    s.trim_end_matches("ki").parse::<u64>().ok().map(|v| v * 1024)
                } else if s.ends_with("mi") {
                    s.trim_end_matches("mi").parse::<u64>().ok().map(|v| v * 1024 * 1024)
                } else if s.ends_with("gi") {
                    s.trim_end_matches("gi").parse::<u64>().ok().map(|v| v * 1024 * 1024 * 1024)
                } else {
                    s.parse::<u64>().ok()
                }
            })
    };

    let parse_pods = |map: Option<&std::collections::BTreeMap<String, k8s_openapi::apimachinery::pkg::api::resource::Quantity>>| {
        map.and_then(|m| m.get("pods"))
            .and_then(|q| q.0.parse::<u32>().ok())
    };

    let cpu_capacity_cores = parse_cpu(capacity);
    let memory_capacity_bytes = parse_mem(capacity);
    let pod_capacity = parse_pods(capacity);
    let ephemeral_storage_capacity_bytes = parse_storage(capacity);

    let cpu_allocatable_cores = parse_cpu(allocatable);
    let memory_allocatable_bytes = parse_mem(allocatable);
    let pod_allocatable = parse_pods(allocatable);
    let ephemeral_storage_allocatable_bytes = parse_storage(allocatable);

    // Determine readiness
    let ready = status
        .and_then(|s| s.conditions.as_ref())
        .and_then(|conds| {
            conds.iter()
                .find(|c| c.type_ == "Ready")
                .map(|c| c.status == "True")
        });

    // Serialize taints, labels, annotations
    let taints = spec
        .and_then(|s| s.taints.as_ref())
        .map(|t| {
            t.iter()
                .map(|taint| {
                    format!(
                        "{}={} ({})",
                        taint.key,
                        taint.value.as_deref().unwrap_or(""),
                        taint.effect
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        });

    let label = metadata
        .labels
        .as_ref()
        .map(|l| serde_json::to_string(l).unwrap_or_default());

    let annotation = metadata
        .annotations
        .as_ref()
        .map(|a| serde_json::to_string(a).unwrap_or_default());

    // Images
    let (image_count, image_names, image_total_size_bytes) = status
        .and_then(|s| s.images.as_ref())
        .map(|imgs| {
            let count = imgs.len() as u32;
            let names = imgs
                .iter()
                .flat_map(|i| i.names.clone().unwrap_or_default())
                .collect::<Vec<_>>();
            let total_size = imgs
                .iter()
                .filter_map(|i| i.size_bytes.map(|s| s as u64))
                .sum::<u64>();
            (Some(count), Some(names), Some(total_size))
        })
        .unwrap_or((None, None, None));

    Ok(InfoNodeEntity {
        node_name: metadata.name.clone(),
        node_uid: metadata.uid.clone(),
        creation_timestamp,
        resource_version: metadata.resource_version.clone().map(|v| v.parse().ok()).flatten(),
        hostname,
        internal_ip,
        architecture,
        os_image,
        kernel_version,
        kubelet_version,
        container_runtime,
        operating_system,
        cpu_capacity_cores,
        memory_capacity_bytes,
        pod_capacity,
        ephemeral_storage_capacity_bytes,
        cpu_allocatable_cores,
        memory_allocatable_bytes,
        ephemeral_storage_allocatable_bytes,
        pod_allocatable,
        ready,
        taints,
        label,
        annotation,
        image_count,
        image_names,
        image_total_size_bytes,
        last_updated_info_at,
        ..Default::default()
    })
}

/// Stub: Convert k8s-openapi Pod to InfoPodEntity
/// TODO: Implement full mapping logic
pub fn map_pod_to_info_entity(_pod: &Pod) -> Result<InfoPodEntity> {
    Ok(InfoPodEntity::default())
}

/// Stub: Convert k8s-openapi Deployment to InfoDeploymentEntity
/// TODO: Implement full mapping logic
pub fn map_deployment_to_info_entity(_deployment: &Deployment) -> Result<InfoDeploymentEntity> {
    Ok(InfoDeploymentEntity::default())
}

/// Stub: Convert k8s-openapi Namespace to InfoNamespaceEntity
/// TODO: Implement full mapping logic
pub fn map_namespace_to_info_entity(_namespace: &Namespace) -> Result<InfoNamespaceEntity> {
    Ok(InfoNamespaceEntity::default())
}
