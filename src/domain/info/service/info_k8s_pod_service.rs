use std::collections::HashSet;

use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use tracing::debug;
use validator::Validate;

use crate::api::dto::k8s_pod_query_request_dto::K8sPodQueryRequestDto;
use crate::api::dto::paginated_response::PaginatedResponse;
use crate::app_state::AppState;
use crate::core::client::kube_client::build_kube_client;
use crate::core::client::mappers::map_pod_to_info_entity;
use crate::core::client::pods::{fetch_pod_by_name_and_namespace, fetch_pod_by_uid};
use crate::core::persistence::info::k8s::pod::info_pod_api_repository_trait::InfoPodApiRepository;
use crate::core::persistence::info::k8s::pod::info_pod_entity::InfoPodEntity;
use crate::core::persistence::info::k8s::pod::info_pod_repository::InfoPodRepository;
use crate::core::state::runtime::k8s::k8s_runtime_state::RuntimePod;
use crate::core::state::runtime::k8s::k8s_runtime_state_repository_trait::K8sRuntimeStateRepositoryTrait;
use crate::domain::info::dto::info_k8s_pod_patch_request::InfoK8sPodPatchRequest;

pub async fn get_info_k8s_pod(pod_uid: String) -> Result<InfoPodEntity> {
    let repo = InfoPodRepository::new();

    if let Ok(existing) = repo.read(&pod_uid) {
        if let Some(ts) = existing.last_updated_info_at {
            if Utc::now().signed_duration_since(ts) <= Duration::hours(1) {
                debug!("Using cached pod info for '{pod_uid}'");
                return Ok(existing);
            }
        }

        if let (Some(ns), Some(name)) = (existing.namespace.clone(), existing.pod_name.clone()) {
            debug!("Refreshing pod info for '{pod_uid}' via {ns}/{name}");
            let kube_client = build_kube_client().await?;
            let pod = fetch_pod_by_name_and_namespace(&kube_client, &ns, &name).await?;

            let mut updated = map_pod_to_info_entity(&pod)?;
            updated.last_updated_info_at = Some(Utc::now());
            updated.pod_uid = Some(pod_uid.clone());
            repo.update(&updated)?;

            return Ok(updated);
        }

        debug!("Missing namespace or pod_name for '{pod_uid}', returning cached record");
        return Ok(existing);
    }

    debug!("No cache found; fetching pod '{pod_uid}' by UID directly");
    let kube_client = build_kube_client().await?;
    let pod = fetch_pod_by_uid(&kube_client, &pod_uid).await?;
    let mut entity = map_pod_to_info_entity(&pod)?;
    entity.last_updated_info_at = Some(Utc::now());
    entity.pod_uid = Some(pod_uid.clone());
    repo.insert(&entity)?;

    Ok(entity)
}

fn intersect(prev: Option<HashSet<String>>, new_list: &Vec<String>) -> HashSet<String> {
    let new_set: HashSet<String> = new_list.iter().cloned().collect();

    match prev {
        Some(old) => old.intersection(&new_set).cloned().collect(),
        None => new_set,
    }
}

pub async fn load_pod_entities(uids: &[String], state: AppState) -> Result<Vec<InfoPodEntity>> {
    let repo = InfoPodRepository::new();
    let runtime = state.k8s_state.repo.get().await;

    let mut result = Vec::new();
    let mut stale_exists = false;

    for uid in uids {
        if let Ok(entity) = repo.read(uid) {
            if let Some(ts) = entity.last_updated_info_at {
                if Utc::now().signed_duration_since(ts) <= Duration::hours(1) {
                    result.push(entity);
                    continue;
                }
            }
        }
        stale_exists = true;
    }

    if !stale_exists {
        return Ok(result);
    }

    let client = build_kube_client().await?;

    for uid in uids {
        let Some(rpod) = runtime.pods.get(uid) else {
            debug!("Runtime pod missing for uid {uid}");
            continue;
        };

        let pod = fetch_pod_by_name_and_namespace(&client, &rpod.namespace, &rpod.name).await?;
        let mut mapped = map_pod_to_info_entity(&pod)?;
        mapped.last_updated_info_at = Some(Utc::now());
        mapped.pod_uid = mapped.pod_uid.or_else(|| Some(uid.clone()));

        let mut entity = match repo.read(uid) {
            Ok(mut existing) => {
                existing.merge_from(mapped);
                existing
            }
            Err(_) => mapped,
        };

        entity.last_updated_info_at = Some(Utc::now());

        if let Err(err) = repo.update(&entity) {
            debug!("Update failed for pod {uid}, attempting insert: {err:?}");
            repo.insert(&entity)?;
        }

        result.push(entity);
    }

    Ok(result)
}

pub fn apply_additional_filters(
    pods: Vec<InfoPodEntity>,
    filter: &K8sPodQueryRequestDto,
) -> Vec<InfoPodEntity> {
    pods.into_iter()
        .filter(|p| {
            if let Some(label_selector) = &filter.label_selector {
                let sel = label_selector.to_lowercase();
                let labels = p
                    .label
                    .as_ref()
                    .map(|l| l.to_lowercase())
                    .unwrap_or_default();
                if !labels.contains(&sel) {
                    return false;
                }
            }

            if let Some(team) = &filter.team {
                if p.team.as_deref() != Some(team.as_str()) {
                    return false;
                }
            }

            if let Some(start) = filter.start {
                if let Some(ts) = p.last_updated_info_at {
                    if ts.naive_utc() < start {
                        return false;
                    }
                }
            }

            if let Some(end) = filter.end {
                if let Some(ts) = p.last_updated_info_at {
                    if ts.naive_utc() > end {
                        return false;
                    }
                }
            }

            true
        })
        .collect()
}

pub fn sort_and_paginate(
    mut pods: Vec<InfoPodEntity>,
    filter: &K8sPodQueryRequestDto,
) -> PaginatedResponse<InfoPodEntity> {
    let total = pods.len();

    if let Some(sort) = &filter.sort {
        match sort.as_str() {
            "name_asc" => pods.sort_by(|a, b| a.pod_name.cmp(&b.pod_name)),
            "name_desc" => pods.sort_by(|a, b| b.pod_name.cmp(&a.pod_name)),
            "node_asc" => pods.sort_by(|a, b| a.node_name.cmp(&b.node_name)),
            "node_desc" => pods.sort_by(|a, b| b.node_name.cmp(&a.node_name)),
            _ => {}
        }
    }

    let limit = filter.limit.unwrap_or(50);
    let offset = filter.offset.unwrap_or(0);

    let items = pods.into_iter().skip(offset).take(limit).collect();

    PaginatedResponse {
        items,
        total,
        limit,
        offset,
    }
}

pub async fn list_k8s_pod_uids(
    state: AppState,
    filter: &K8sPodQueryRequestDto,
) -> Vec<String> {
    let s = state.k8s_state.repo.get().await;

    let mut candidates: Option<HashSet<String>> = None;

    if let Some(ns) = &filter.namespace {
        candidates = s
            .pods_by_namespace
            .get(ns)
            .map(|uids| uids.iter().cloned().collect())
    }

    if let Some(node) = &filter.node {
        candidates = s
            .pods_by_node
            .get(node)
            .map(|uids| intersect(candidates, uids));
    }

    if let Some(dep) = &filter.deployment {
        candidates = s
            .pods_by_deployment
            .get(dep)
            .map(|uids| intersect(candidates, uids));
    }

    let pods: Vec<&RuntimePod> = match candidates {
        Some(set) => set.into_iter().filter_map(|uid| s.pods.get(&uid)).collect(),
        None => s.pods.values().collect(),
    };

    pods.into_iter()
        .filter(|p| match &filter.name {
            Some(n) => p.name.contains(n),
            None => true,
        })
        .map(|p| p.uid.clone())
        .collect()
}

pub async fn list_k8s_pods(
    state: AppState,
    filter: K8sPodQueryRequestDto,
) -> Result<PaginatedResponse<InfoPodEntity>> {
    let uids = list_k8s_pod_uids(state.clone(), &filter).await;
    let entities = load_pod_entities(&uids, state).await?;
    let entities = apply_additional_filters(entities, &filter);

    Ok(sort_and_paginate(entities, &filter))
}

pub async fn patch_info_k8s_pod(
    id: String,
    patch: InfoK8sPodPatchRequest,
) -> Result<serde_json::Value> {
    patch.validate()?;
    let repo = InfoPodRepository::new();

    let mut entity = repo
        .read(&id)
        .map_err(|_| anyhow!("Pod '{}' not found", id))?;

    if let Some(team) = patch.team {
        entity.team = Some(team);
    }

    if let Some(service) = patch.service {
        entity.service = Some(service);
    }

    if let Some(env) = patch.env {
        entity.env = Some(env);
    }

    entity.last_updated_info_at = Some(Utc::now());

    repo.update(&entity)?;

    Ok(serde_json::to_value(&entity)?)
}
