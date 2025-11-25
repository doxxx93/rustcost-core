use anyhow::{anyhow, Context, Result};
use chrono::{Duration, Utc};
use serde_json::to_string_pretty;
use tracing::debug;

use crate::api::dto::info_dto::K8sListQuery;
use crate::core::client::k8s::client_k8s_container_mapper::map_container_status_to_info_container_entity;
use crate::core::client::k8s::client_k8s_pod::{fetch_pod_by_name_and_namespace, fetch_pods, fetch_pods_by_namespace, fetch_pods_by_node};
use crate::core::client::k8s::util::{build_client, read_token};
use crate::core::persistence::info::k8s::container::info_container_api_repository_trait::InfoContainerApiRepository;
use crate::core::persistence::info::k8s::container::info_container_entity::InfoContainerEntity;
use crate::core::persistence::info::k8s::container::info_container_repository::InfoContainerRepository;
use crate::core::persistence::info::path::info_k8s_container_dir_path;
use crate::domain::info::dto::info_k8s_container_patch_request::InfoK8sContainerPatchRequest;
use std::fs;
use validator::Validate;

/// Fetch one container info by its unique ID, with cache + refresh if stale.
pub async fn get_info_k8s_container(container_id: String) -> Result<InfoContainerEntity> {
    let repo = InfoContainerRepository::new();

    // 1️⃣ Try reading existing entity from repo
    if let Ok(existing) = repo.read(&container_id) {
        if let Some(ts) = existing.last_updated_info_at {
            if Utc::now().signed_duration_since(ts) <= Duration::hours(1) {
                debug!("✅ Using cached container info for '{}'", container_id);
                return Ok(existing);
            }
        }

        // Cached but expired — refresh via API
        if let (Some(ns), Some(pod_name), Some(container_name)) = (
            existing.namespace.clone(),
            existing.pod_uid.clone(),
            existing.container_name.clone(),
        ) {
            debug!("🔄 Cache expired; fetching fresh container info for '{}'", container_id);

            let token = read_token()?;
            let client = build_client()?;

            let pod = fetch_pod_by_name_and_namespace(&token, &client, &ns, &pod_name).await?;
            let _status = pod.status
                .as_ref()
                .and_then(|s| s.container_statuses.as_ref().and_then(|cs| cs.iter().find(|c| c.name == container_name)));

            let _spec = pod.spec
                .as_ref()
                .and_then(|s| s.containers.iter().find(|c| c.name == container_name))
                .ok_or_else(|| anyhow!("Container '{}' not found", container_name))?;

            // TODO: Implement proper container mapping
            let mut updated_entity = InfoContainerEntity::default();

            updated_entity.last_updated_info_at = Some(Utc::now());
            updated_entity.container_id = Some(container_id.clone());

            debug!("🧩 Updated InfoContainerEntity for '{}': {}", container_id, to_string_pretty(&updated_entity)?);

            repo.update(&updated_entity)?;
            return Ok(updated_entity);
        } else {
            debug!(
                "⚠️ Missing namespace/pod/container name for '{}', cannot refresh.",
                container_id
            );
            return Ok(existing);
        }
    }

    // 2️⃣ No cache found → fetch directly (requires identifiers)
    debug!(
        "🔍 No cache found; cannot fetch container '{}' without namespace/pod/container name",
        container_id
    );

    Err(anyhow!(
        "Missing namespace, pod name, or container name to fetch container '{}'",
        container_id
    ))
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
