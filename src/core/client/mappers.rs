/// Maps kube-rs / k8s-openapi types → internal domain models
use crate::core::client::kube_resources::{Node, Pod, Deployment, Namespace};
use crate::core::persistence::info::k8s::node::info_node_entity::InfoNodeEntity;
use crate::core::persistence::info::k8s::pod::info_pod_entity::InfoPodEntity;
use crate::core::persistence::info::k8s::deployment::info_deployment_entity::InfoDeploymentEntity;
use crate::core::persistence::info::k8s::namespace::info_namespace_entity::InfoNamespaceEntity;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashSet};
use std::convert::TryFrom;

/// Converts a k8s-openapi Node object into an InfoNodeEntity
pub fn map_node_to_info_entity(node: &Node, now: DateTime<Utc>) -> Result<InfoNodeEntity> {
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

    let last_updated_info_at = Some(now);

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
pub fn map_pod_to_info_entity(pod: &Pod) -> Result<InfoPodEntity> {
    let metadata = &pod.metadata;
    let spec = pod.spec.as_ref();
    let status = pod.status.as_ref();

    let creation_timestamp = metadata.creation_timestamp.as_ref().map(|t| t.0);
    let start_time = status.and_then(|s| s.start_time.as_ref().map(|t| t.0));

    let pod_uid = metadata.uid.clone();
    let pod_name = metadata.name.clone();
    let namespace = metadata.namespace.clone();
    let resource_version = metadata.resource_version.clone();

    let node_name = spec.and_then(|s| s.node_name.clone());
    let host_ip = status.and_then(|s| s.host_ip.clone());
    let pod_ip = status
        .and_then(|s| s.pod_ip.clone().or_else(|| {
            s.pod_ips
                .as_ref()
                .and_then(|ips| ips.first().map(|p| p.ip.clone()))
        }));

    let qos_class = status.and_then(|s| s.qos_class.clone());
    let phase = status.and_then(|s| s.phase.clone());
    let ready = status
        .and_then(|s| s.conditions.as_ref())
        .and_then(|conds| conds.iter().find(|c| c.type_ == "Ready"))
        .map(|c| c.status == "True");

    let restart_count = status
        .and_then(|s| s.container_statuses.as_ref())
        .map(|statuses| {
            statuses
                .iter()
                .map(|cs| cs.restart_count.max(0) as u32)
                .sum::<u32>()
        });

    let (owner_kind, owner_name, owner_uid) = metadata
        .owner_references
        .as_ref()
        .and_then(|owners| owners.first())
        .map(|owner| {
            (
                Some(owner.kind.clone()),
                Some(owner.name.clone()),
                Some(owner.uid.clone()),
            )
        })
        .unwrap_or((None, None, None));

    let container_count = spec.map(|s| s.containers.len() as u32);
    let container_names = spec
        .map(|s| s.containers.iter().map(|c| c.name.clone()).collect::<Vec<_>>())
        .filter(|v| !v.is_empty());
    let container_images = spec
        .map(|s| {
            s.containers
                .iter()
                .filter_map(|c| c.image.clone())
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty());
    let container_ports = spec
        .map(|s| {
            s.containers
                .iter()
                .flat_map(|c| c.ports.as_ref().into_iter().flatten())
                .filter_map(|p| u16::try_from(p.container_port).ok())
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty());
    let restart_policy = spec.and_then(|s| s.restart_policy.clone());
    let scheduler_name = spec.and_then(|s| s.scheduler_name.clone());
    let service_account = spec.and_then(|s| s.service_account_name.clone());

    let container_ids = status
        .and_then(|s| s.container_statuses.as_ref())
        .map(|statuses| {
            statuses
                .iter()
                .filter_map(|cs| cs.container_id.clone())
                .filter(|id: &String| !id.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty());

    let image_ids = status
        .and_then(|s| s.container_statuses.as_ref())
        .map(|statuses| {
            statuses
                .iter()
                .map(|cs| cs.image_id.clone())
                .filter(|id: &String| !id.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty());

    let container_started_at = status
        .and_then(|s| s.container_statuses.as_ref())
        .map(|statuses| {
            statuses
                .iter()
                .filter_map(|cs| {
                    cs.state
                        .as_ref()
                        .and_then(|state| state.running.as_ref())
                        .and_then(|running| running.started_at.as_ref())
                        .map(|ts| ts.0)
                })
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty());

    let volume_names = spec
        .and_then(|s| s.volumes.as_ref())
        .map(|vols| vols.iter().map(|v| v.name.clone()).collect::<Vec<_>>())
        .filter(|v| !v.is_empty());
    let volume_count = volume_names.as_ref().map(|v| v.len() as u32);
    let pvc_names = spec
        .and_then(|s| s.volumes.as_ref())
        .map(|vols| {
            vols.iter()
                .filter_map(|v| v.persistent_volume_claim.as_ref())
                .map(|pvc| pvc.claim_name.clone())
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty());
    let mount_paths = spec
        .map(|s| {
            let mut seen = HashSet::new();
            let mut mounts = Vec::new();
            for container in &s.containers {
                if let Some(volume_mounts) = &container.volume_mounts {
                    for mount in volume_mounts {
                        if seen.insert(mount.mount_path.clone()) {
                            mounts.push(mount.mount_path.clone());
                        }
                    }
                }
            }
            mounts
        })
        .filter(|v| !v.is_empty());
    let termination_grace_period_seconds = spec
        .and_then(|s| s.termination_grace_period_seconds)
        .and_then(|v| u32::try_from(v).ok());
    let tolerations = spec
        .and_then(|s| s.tolerations.as_ref())
        .map(|tolerations| {
            tolerations
                .iter()
                .map(format_toleration)
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty());

    let label = metadata.labels.as_ref().and_then(flatten_map);
    let annotation = metadata.annotations.as_ref().and_then(flatten_map);

    Ok(InfoPodEntity {
        pod_name,
        namespace,
        pod_uid,
        creation_timestamp,
        start_time,
        resource_version,
        last_updated_info_at: None,
        deleted: None,
        last_check_deleted_count: None,
        node_name,
        host_ip,
        pod_ip,
        qos_class,
        phase,
        ready,
        restart_count,
        owner_kind,
        owner_name,
        owner_uid,
        container_count,
        container_names,
        container_images,
        container_ids,
        container_started_at,
        image_ids,
        container_ports,
        restart_policy,
        scheduler_name,
        service_account,
        volume_count,
        volume_names,
        pvc_names,
        mount_paths,
        termination_grace_period_seconds,
        tolerations,
        label,
        annotation,
        team: None,
        service: None,
        env: None,
    })
}

fn flatten_map(map: &BTreeMap<String, String>) -> Option<String> {
    if map.is_empty() {
        return None;
    }

    Some(
        map.iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(","),
    )
}

fn format_toleration(t: &k8s_openapi::api::core::v1::Toleration) -> String {
    let mut parts = Vec::new();

    if let Some(key) = &t.key {
        parts.push(key.clone());
    }

    if let Some(op) = &t.operator {
        parts.push(op.clone());
    }

    if let Some(value) = &t.value {
        parts.push(value.clone());
    }

    if let Some(effect) = &t.effect {
        parts.push(effect.clone());
    }

    if let Some(seconds) = t.toleration_seconds {
        parts.push(format!("{seconds}s"));
    }

    parts.join(":")
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
