use crate::core::client::kube_client::build_kube_client;
use crate::core::client::mappers::map_node_to_info_entity;
use crate::core::client::nodes::{fetch_node_by_name, fetch_nodes};
use crate::core::persistence::info::k8s::node::info_node_api_repository_trait::InfoNodeApiRepository;
use crate::core::persistence::info::k8s::node::info_node_entity::InfoNodeEntity;
use crate::core::persistence::info::k8s::node::info_node_repository::InfoNodeRepository;
use crate::core::persistence::info::path::info_k8s_node_dir_path;
use crate::api::dto::info_dto::K8sListNodeQuery;
use crate::domain::info::dto::info_k8s_node_patch_request::{
    InfoK8sNodePatchRequest,
    InfoK8sNodePricePatchRequest,
};
use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use serde_json::Map;
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
        debug!(
            "Node '{}' info is missing or stale – refreshing from K8s API",
            node_name
        );

        // Build K8s client
        let client = build_kube_client().await?;

        // Fetch from K8s API
        let node = fetch_node_by_name(&client, &node_name).await?;
        let updated_entity = map_node_to_info_entity(&node, now)?;

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
pub async fn list_k8s_nodes(filter: K8sListNodeQuery) -> Result<Vec<InfoNodeEntity>> {
    let now = Utc::now();
    debug!("Listing all Kubernetes nodes");

    let client = build_kube_client().await?;
    let repo = InfoNodeRepository::new();

    let mut cached_entities = Vec::new();
    let mut expired_or_missing = false;

    // 1) Load local cache
    let node_dir = info_k8s_node_dir_path();
    if node_dir.exists() {
        if let Ok(entries) = fs::read_dir(&node_dir) {
            for entry in entries.flatten() {
                let node_name = entry.file_name().to_string_lossy().to_string();

                if let Ok(existing) = repo.read(&node_name) {
                    if let Some(ts) = existing.last_updated_info_at {
                        if now.signed_duration_since(ts) <= Duration::hours(1) {
                            debug!("Using cached node info for '{}'", node_name);
                            cached_entities.push(existing);
                            continue;
                        }
                    }
                }

                debug!("Cache expired or missing for '{}'", node_name);
                expired_or_missing = true;
            }
        }
    }

    // 2) If cache is valid for all records → return only cached
    if !expired_or_missing && !cached_entities.is_empty() {
        debug!("All cached node info is fresh, skipping API call.");
        return Ok(apply_node_filters(cached_entities, &filter));
    }

    // 3) Fetch from Kubernetes API
    debug!("Fetching nodes from K8s API (some cache expired or missing)");
    let node_list = fetch_nodes(&client).await?;
    debug!("Fetched {} node(s) from API", node_list.len());

    let mut result_entities = cached_entities;

    // 4) Process each node
    for node in node_list {
        let node_name = node.metadata.name.clone().unwrap_or_default();

        // Map API → entity
        let mapped = map_node_to_info_entity(&node, now)?;

        // If cache exists → merge
        let merged = if let Ok(mut existing) = repo.read(&node_name) {
            existing.merge_from(mapped);
            existing
        } else {
            mapped
        };

        // Save merged result
        if let Err(e) = repo.update(&merged) {
            debug!("Failed to update node '{}': {:?}", &node_name, e);
        }

        result_entities.push(merged);
    }

    Ok(apply_node_filters(result_entities, &filter))
}

fn apply_node_filters(
    nodes: Vec<InfoNodeEntity>,
    filter: &K8sListNodeQuery,
) -> Vec<InfoNodeEntity> {
    nodes
        .into_iter()
        .filter(|n| {
            if let Some(selector) = &filter.label_selector {
                if !matches_node_label(n, selector) {
                    return false;
                }
            }

            if let Some(team) = &filter.team {
                if n.team.as_deref() != Some(team.as_str()) {
                    return false;
                }
            }

            if let Some(service) = &filter.service {
                if n.service.as_deref() != Some(service.as_str()) {
                    return false;
                }
            }

            if let Some(env) = &filter.env {
                if n.env.as_deref() != Some(env.as_str()) {
                    return false;
                }
            }

            true
        })
        .collect()
}

fn matches_node_label(node: &InfoNodeEntity, selector: &str) -> bool {
    let label_json = match &node.label {
        Some(l) => l,
        None => return false,
    };

    // Try to parse stored JSON map {"k":"v",...}
    if let Ok(map) = serde_json::from_str::<Map<String, serde_json::Value>>(label_json) {
        for part in selector.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            if let Some((k, v)) = part.split_once('=') {
                let matches = map
                    .get(k)
                    .and_then(|v0| v0.as_str())
                    .map(|s| s == v)
                    .unwrap_or(false);
                if !matches {
                    return false;
                }
            } else if !map.contains_key(part) {
                return false;
            }
        }
        return true;
    }

    // Fallback to substring match when JSON parse fails
    label_json.to_lowercase().contains(&selector.to_lowercase())
}

pub async fn patch_info_k8s_node_filter(
    id: String,
    patch: InfoK8sNodePatchRequest,
) -> Result<serde_json::Value> {
    patch.validate()?;
    let repo = InfoNodeRepository::new();

    // 1) Load existing record
    let mut entity = repo
        .read(&id)
        .map_err(|_| anyhow!("Node '{}' not found", id))?;

    // 2) Apply patch – only update fields that are Some()
    if let Some(team) = patch.team {
        entity.team = Some(team);
    }

    if let Some(service) = patch.service {
        entity.service = Some(service);
    }

    if let Some(env) = patch.env {
        entity.env = Some(env);
    }

    // 3) Update timestamp
    entity.last_updated_info_at = Some(Utc::now());

    // 4) Store back
    repo.update(&entity)?;

    // 5) Return updated JSON
    Ok(serde_json::to_value(&entity)?)
}

pub async fn patch_info_k8s_node_price(
    id: String,
    patch: InfoK8sNodePricePatchRequest,
) -> Result<serde_json::Value> {
    patch.validate()?;
    let repo = InfoNodeRepository::new();

    // 1) Load existing record
    let mut entity = repo
        .read(&id)
        .map_err(|_| anyhow!("Node '{}' not found", id))?;

    // 2) Apply patch – only update fields that are Some()
    if let Some(fixed_instance_usd) = patch.fixed_instance_usd {
        entity.fixed_instance_usd = Some(fixed_instance_usd);
    }

    if let Some(price_period) = patch.price_period {
        entity.price_period = Some(price_period);
    }

    // 3) Update timestamp
    entity.last_updated_info_at = Some(Utc::now());

    // 4) Store back
    repo.update(&entity)?;

    // 5) Return updated JSON
    Ok(serde_json::to_value(&entity)?)
}
