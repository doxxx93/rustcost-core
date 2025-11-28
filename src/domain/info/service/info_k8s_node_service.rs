use crate::core::client::kube_client::build_kube_client;
use crate::core::client::nodes::{fetch_node_by_name, fetch_nodes};
use crate::core::client::mappers::map_node_to_info_entity;
use crate::core::persistence::info::k8s::node::info_node_api_repository_trait::InfoNodeApiRepository;
use crate::core::persistence::info::k8s::node::info_node_entity::InfoNodeEntity;
use crate::core::persistence::info::k8s::node::info_node_repository::InfoNodeRepository;
use crate::core::persistence::info::path::info_k8s_node_dir_path;
use crate::domain::info::dto::info_k8s_node_patch_request::InfoK8sNodePatchRequest;
use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use std::fs;
use tracing::debug;
use validator::Validate;

pub async fn get_info_k8s_node(node_name: String) -> Result<InfoNodeEntity> {
    let now = Utc::now();
    let repo = InfoNodeRepository::new();

    // Load existing entity
    let entity = repo.read(&node_name)?;

    let needs_refresh = match entity.last_updated_info_at {
        None => true,
        Some(last) => now.signed_duration_since(last) > Duration::hours(1),
    };

    if needs_refresh {
        debug!("Node '{}' info is missing or stale — refreshing from K8s API", node_name);

        // Build K8s client
        let client = build_kube_client().await?;

        // Fetch from K8s API
        let node = fetch_node_by_name(&client, &node_name).await?;
        let updated_entity = map_node_to_info_entity(&node)?;

        // Save refreshed info
        repo.update(&updated_entity)?;

        debug!(
            "Updated node '{}' info successfully (last_updated_info_at = {})",
            node_name, now
        );

        Ok(updated_entity)
    } else {
        debug!(
            "Node '{}' info is up-to-date (last_updated_info_at = {:?})",
            node_name, entity.last_updated_info_at
        );
        Ok(entity)
    }
}


/// List all Kubernetes nodes, using local cache when fresh.
/// Refresh occurs if cache is missing or older than 1 hour.
pub async fn list_k8s_nodes() -> Result<Vec<InfoNodeEntity>> {

    let now = Utc::now();
    debug!("Listing all Kubernetes nodes");

    let client = build_kube_client().await?;
    let repo = InfoNodeRepository::new();

    let mut cached_entities = Vec::new();
    let mut expired_or_missing = false;

    // 1️⃣ Load local cache
    let node_dir = info_k8s_node_dir_path();
    if node_dir.exists() {
        if let Ok(entries) = fs::read_dir(&node_dir) {
            for entry in entries.flatten() {
                let node_name = entry.file_name().to_string_lossy().to_string();

                if let Ok(existing) = repo.read(&node_name) {
                    if let Some(ts) = existing.last_updated_info_at {
                        if Utc::now().signed_duration_since(ts) <= Duration::hours(1) {
                            debug!("✅ Using cached node info for '{}'", node_name);
                            cached_entities.push(existing);
                            continue;
                        }
                    }
                }

                debug!("⚠️ Cache expired or missing for '{}'", node_name);
                expired_or_missing = true;
            }
        }
    }

    // 2️⃣ If cache is valid for all records → return only cached
    if !expired_or_missing && !cached_entities.is_empty() {
        debug!("📦 All cached node info is fresh, skipping API call.");
        return Ok(cached_entities);
    }

    // 3️⃣ Fetch from Kubernetes API
    debug!("🌐 Fetching nodes from K8s API (some cache expired or missing)");
    let node_list = fetch_nodes(&client).await?;
    debug!("Fetched {} node(s) from API", node_list.len());

    let mut result_entities = cached_entities;

    // 4️⃣ Process each node
    for node in node_list {
        let node_name = node.metadata.name.clone().unwrap_or_default();

        // Map API → entity
        let mapped = map_node_to_info_entity(&node)?;

        // If cache exists → merge
        let merged = if let Ok(mut existing) = repo.read(&node_name) {
            existing.merge_from(mapped);
            existing
        } else {
            mapped
        };

        // Save merged result
        if let Err(e) = repo.update(&merged) {
            debug!("⚠️ Failed to update node '{}': {:?}", &node_name, e);
        }

        result_entities.push(merged);
    }

    Ok(result_entities)
}


pub async fn patch_info_k8s_node(
    id: String,
    patch: InfoK8sNodePatchRequest,
) -> Result<serde_json::Value> {
    patch.validate()?;
    let repo = InfoNodeRepository::new();

    // 1️⃣ Load existing record
    let mut entity = repo
        .read(&id)
        .map_err(|_| anyhow!("Node '{}' not found", id))?;

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
