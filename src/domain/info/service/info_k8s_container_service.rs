use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use tracing::{debug, error, warn};

use crate::api::dto::info_dto::K8sListQuery;
use crate::core::client::k8s::client_k8s_pod::{fetch_pods, fetch_pods_by_namespace, fetch_pods_by_node};
use crate::core::client::k8s::util::{build_client, read_token};
use crate::core::persistence::info::k8s::container::info_container_api_repository_trait::InfoContainerApiRepository;
use crate::core::persistence::info::k8s::container::info_container_entity::InfoContainerEntity;
use crate::core::persistence::info::k8s::container::info_container_repository::InfoContainerRepository;
use crate::core::persistence::info::path::info_k8s_container_dir_path;
use crate::domain::info::dto::info_k8s_container_patch_request::InfoK8sContainerPatchRequest;
use std::fs;
use k8s_openapi::api::core::v1::{ContainerStatus, Pod};
use kube::Api;
use validator::Validate;
use crate::core::client::kube_client::build_kube_client;

/// Fetch one container info by its unique ID, with cache + refresh if stale.
pub async fn get_info_k8s_container(container_id: String) -> Result<InfoContainerEntity> {
    let repo = InfoContainerRepository::new();

    // ---- 1. Load from cache ----
    let mut entity = repo
        .read(&container_id)
        .context("Failed to read container from cache")?;

    // ---- 2. Cache freshness check ----
    if cache_is_fresh(entity.creation_timestamp, entity.last_updated_info_at) {
        debug!("Using fresh cached container '{}'", container_id);
        return Ok(entity);
    }

    debug!("Cache expired or missing for '{}', refreshing...", container_id);

    // ---- 3. Extract identity fields (needed for API call) ----
    let ns = entity.namespace.clone();
    let pod_name = entity.pod_name.clone();
    let container_name = entity.container_name.clone();

    // If ANY are missing, we cannot query API
    let identity_valid = ns.is_some() && pod_name.is_some() && container_name.is_some();

    let ns = ns.unwrap_or_default();
    let pod_name = pod_name.unwrap_or_default();
    let container_name = container_name.unwrap_or_default();

    // ---- 4. Query Kubernetes API ----
    let client = build_kube_client().await?;
    let pod_api: Api<Pod> = Api::namespaced(client, &ns);

    let pod_result = pod_api.get(&pod_name).await;

    match pod_result {
        // ---- 4A. POD FOUND ----
        Ok(pod) => {
            let updated = map_container_from_pod(&pod, &container_name)
                .context("Container not found in pod")?;

            repo.update(&updated)?;
            Ok(updated)
        }

        // ---- 4B. 404 NOT FOUND ----
        Err(kube::Error::Api(ae)) if ae.code == 404 => {
            warn!(
                "Pod '{}' in ns '{}' not found (404). Container '{}' is possibly deleted.",
                pod_name, ns, container_id
            );

            // If identity is invalid → "double 404" → return a REAL 404
            if !identity_valid {
                anyhow::bail!(
                    "Pod '{}' in ns '{}' not found and cached entity incomplete -> 404",
                    pod_name,
                    ns
                );
            }

            // Cached entity is valid → mark as deleted
            entity.deleted = Some(true);
            entity.last_check_deleted_count = Some(
                entity.last_check_deleted_count.unwrap_or(0) + 1
            );
            entity.last_updated_info_at = Some(Utc::now());

            // Store deletion
            let _ = repo.update(&entity);

            Ok(entity)
        }

        // ---- 4C. API ERROR (timeout, network, auth...) ----
        Err(e) => {
            error!(
                "API error fetching pod '{}' in ns '{}': {}. Falling back to cached entity.",
                pod_name, ns, e
            );
            Ok(entity)
        }
    }
}


fn cache_is_fresh(
    creation_ts: Option<DateTime<Utc>>,
    last_updated_ts: Option<DateTime<Utc>>,
) -> bool {
    // If we *have* a creation timestamp → do NOT use cache
    // Always refresh from live K8s data
    if creation_ts.is_some() {
        return false;
    }

    // If the last update exists and is recent → use cache
    if let Some(ts) = last_updated_ts {
        let age = Utc::now().signed_duration_since(ts);
        return age <= Duration::hours(1);
    }

    false
}


pub fn map_container_from_pod(pod: &Pod, cname: &str) -> Result<InfoContainerEntity> {
    let metadata = &pod.metadata;

    // --- Container spec lookup ---
    let spec = pod.spec.as_ref().context("missing pod spec")?;
    let container_spec = spec.containers
        .iter()
        .find(|c| c.name == cname)
        .context("container not found in pod spec")?;

    // --- Container status lookup ---
    let status_container: Option<&ContainerStatus> = pod
        .status
        .as_ref()
        .and_then(|st| st.container_statuses.as_ref())
        .and_then(|list| list.iter().find(|c| c.name == cname));

    // --- Extract runtime state ---
    let (state, reason, message, exit_code, restart_count, ready) = if let Some(cs) = status_container {
        let restart_count = cs.restart_count;
        let ready = cs.ready;

        match &cs.state {
            Some(st) => {
                if let Some(_r) = &st.running {
                    ("Running".into(), None, None, None, Some(restart_count), Some(ready))
                } else if let Some(w) = &st.waiting {
                    (
                        "Waiting".into(),
                        w.reason.clone(),
                        w.message.clone(),
                        None,
                        Some(restart_count),
                        Some(ready),
                    )
                } else if let Some(t) = &st.terminated {
                    (
                        "Terminated".into(),
                        t.reason.clone(),
                        t.message.clone(),
                        Some(t.exit_code),
                        Some(restart_count),
                        Some(ready),
                    )
                } else {
                    ("Unknown".into(), None, None, None, Some(restart_count), Some(ready))
                }
            }
            None => ("Unknown".into(), None, None, None, Some(restart_count), Some(ready)),
        }
    } else {
        // No container status at all
        ("Unknown".into(), None, None, None, None, None)
    };

    // --- Container runtime ID ---
    let container_runtime_id = status_container
        .and_then(|cs| cs.container_id.clone())
        .or_else(|| {
            // fallback synthetic
            metadata.uid.clone().map(|uid| format!("{}-{}", uid, cname))
        });

    // --- Image & Image ID ---
    let image = container_spec.image.clone();
    let image_id = status_container.and_then(|cs| Option::from(cs.image_id.clone()));

    // --- Networking: hostIP & podIP ---
    let (host_ip, pod_ip) = pod.status.as_ref()
        .map(|st| (st.host_ip.clone(), st.pod_ip.clone()))
        .unwrap_or((None, None));

    // --- Resource Requests/ Limits ---
    let (cpu_req, mem_req, cpu_limit, mem_limit) = {
        let r = container_spec.resources.as_ref();

        let cpu_req = r
            .and_then(|x| x.requests.as_ref())
            .and_then(|m| m.get("cpu"))
            .and_then(|q| q.0.parse::<u64>().ok());

        let mem_req = r
            .and_then(|x| x.requests.as_ref())
            .and_then(|m| m.get("memory"))
            .and_then(|q| q.0.parse::<u64>().ok());

        let cpu_limit = r
            .and_then(|x| x.limits.as_ref())
            .and_then(|m| m.get("cpu"))
            .and_then(|q| q.0.parse::<u64>().ok());

        let mem_limit = r
            .and_then(|x| x.limits.as_ref())
            .and_then(|m| m.get("memory"))
            .and_then(|q| q.0.parse::<u64>().ok());

        (cpu_req, mem_req, cpu_limit, mem_limit)
    };

    // --- Volume mounts / devices ---
    let volume_mounts = container_spec.volume_mounts
        .as_ref()
        .map(|v| v.iter().map(|m| m.mount_path.clone()).collect());

    let volume_devices = container_spec.volume_devices
        .as_ref()
        .map(|v| v.iter().map(|d| d.device_path.clone()).collect());

    // --- Labels / Annotations ---
    let labels = metadata.labels.as_ref().map(|m| {
        m.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(",")
    });

    let annotations = metadata.annotations.as_ref().map(|m| {
        m.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(",")
    });

    // --- Build final entity ---
    Ok(InfoContainerEntity {
        // Identity
        pod_uid: metadata.uid.clone(),
        pod_name: metadata.name.clone(),   // <-- IMPORTANT: You were missing this
        container_name: Some(container_spec.name.clone()),
        namespace: metadata.namespace.clone(),

        // Lifecycle
        creation_timestamp: metadata.creation_timestamp.as_ref().map(|t| t.0),
        start_time: pod.status.as_ref().and_then(|st| st.start_time.as_ref().map(|t| t.0)),
        container_id: container_runtime_id,
        image,
        image_id,

        // Status
        state: Some(state),
        reason,
        message,
        exit_code,
        restart_count,
        ready,

        // Node association
        node_name: spec.node_name.clone(),
        host_ip,
        pod_ip,

        // Resources
        cpu_request_millicores: cpu_req,
        memory_request_bytes: mem_req,
        cpu_limit_millicores: cpu_limit,
        memory_limit_bytes: mem_limit,

        // Volumes
        volume_mounts,
        volume_devices,

        // Metadata
        labels,
        annotations,

        // Bookkeeping
        last_updated_info_at: Some(Utc::now()),
        deleted: Some(false),
        last_check_deleted_count: Some(0),

        // Team/service/env not set here
        team: None,
        service: None,
        env: None,
    })
}


/// List containers — supports optional filters: namespace, pod_name, node_name.
/// Uses local FS cache when fresh, refreshes stale entries.
pub async fn list_k8s_containers(filter: K8sListQuery) -> Result<Vec<InfoContainerEntity>> {
    let token = read_token()?;
    let client = build_client()?;
    let repo = InfoContainerRepository::new();

    let mut cached_entities = Vec::new();
    let mut expired_or_missing = false;

    // -------------------------------------------------------------
    // 1️⃣ Load cache entries from filesystem
    // -------------------------------------------------------------
    let container_dir = info_k8s_container_dir_path();
    if container_dir.exists() {
        if let Ok(entries) = fs::read_dir(&container_dir) {
            for entry in entries.flatten() {
                let id = entry.file_name().to_string_lossy().to_string();
                if let Ok(existing) = repo.read(&id) {
                    if let Some(ts) = existing.last_updated_info_at {
                        if Utc::now().signed_duration_since(ts) <= Duration::hours(1) {
                            debug!("✅ Using cached container info: {}", id);
                            cached_entities.push(existing);
                            continue;
                        }
                    }

                    debug!("⚠️ Cache expired for container '{}'", id);
                    expired_or_missing = true;
                } else {
                    expired_or_missing = true;
                }
            }
        }
    }

    // -------------------------------------------------------------
    // 2️⃣ If everything is fresh, return cached only
    // -------------------------------------------------------------
    if !expired_or_missing && !cached_entities.is_empty() {
        debug!("📦 All cached containers fresh — no API call needed.");
        return Ok(cached_entities);
    }

    // -------------------------------------------------------------
    // 3️⃣ Fetch pods from API (containers come from Pods)
    // -------------------------------------------------------------
    debug!("🌐 Fetching pods for container refresh");

    let pod_list = if let Some(ns) = &filter.namespace {
        fetch_pods_by_namespace(&token, &client, ns).await?
    } else if let Some(node) = &filter.node_name {
        fetch_pods_by_node(&token, &client, node).await?
    } else {
        fetch_pods(&token, &client).await?
    };

    debug!("Fetched {} pod(s) from API", pod_list.items.len());

    let mut results = cached_entities;

    // -------------------------------------------------------------
    // 4️⃣ Convert pod container statuses into InfoContainerEntity
    // -------------------------------------------------------------
    for pod in pod_list.items {
        let ns = pod.metadata.namespace.clone().unwrap_or_default();
        let pod_uid = pod.metadata.uid.clone().unwrap_or_default();

        if let Some(ref status) = pod.status {
            if let Some(ref spec) = pod.spec {
                for container in spec.containers.iter() {
                    let cname = &container.name;

                    let _cs = status.container_statuses
                        .as_ref()
                        .and_then(|statuses| statuses.iter().find(|s| &s.name == cname));

                    // TODO: Implement proper container mapping
                    let mut mapped = InfoContainerEntity::default();

                mapped.namespace = Some(ns.clone());
                mapped.pod_uid = Some(pod_uid.clone());
                mapped.container_name = Some(cname.clone());
                mapped.container_id = Some(format!("{}-{}", pod_uid, cname));
                mapped.last_updated_info_at = Some(Utc::now());

                let id = mapped.container_id.clone().unwrap();

                // If cached exists → merge metadata
                let merged = if let Ok(mut existing) = repo.read(&id) {
                    existing.merge_from(mapped);
                    existing
                } else {
                    mapped
                };

                // Write back to FS
                if let Err(e) = repo.update(&merged) {
                    debug!("⚠️ Failed to update container '{}': {:?}", id, e);
                }

                results.push(merged);
                }
            }
        }
    }

    Ok(results)
}

pub async fn patch_info_k8s_container(
    id: String,
    patch: InfoK8sContainerPatchRequest,
) -> Result<serde_json::Value> {
    patch.validate()?;
    let repo = InfoContainerRepository::new();

    // 1️⃣ Load existing record
    let mut entity = repo.read(&id)
        .context(format!("Cannot patch container '{}': missing info file", id))?;

    if entity.pod_uid.is_none() || entity.container_name.is_none() {
        return Err(anyhow!("Corrupt entity: missing identifiers"));
    }

    // 2️⃣ Apply patch — only update fields that are Some()
    if let Some(team) = patch.team {
        entity.team = Some(team);
    }

    if let Some(service) = patch.service {
        entity.service = Some(service);
    }

    if let Some(env) = patch.env {
        entity.env = Some(env);
    }

    // 3️⃣ Update timestamp
    entity.last_updated_info_at = Some(Utc::now());

    // 4️⃣ Store back
    repo.update(&entity)?;

    // 5️⃣ Return updated JSON
    Ok(serde_json::to_value(&entity)?)
}
