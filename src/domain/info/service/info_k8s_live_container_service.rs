use anyhow::{anyhow, Result};
use chrono::Utc;
use crate::api::dto::paginated_response::PaginatedResponse;
use crate::core::client::kube_client::build_kube_client;
use crate::core::client::pods::fetch_pods;
use crate::core::persistence::info::k8s::container::info_container_entity::InfoContainerEntity;
use crate::domain::info::service::info_k8s_container_service::map_container_from_pod;

pub async fn get_k8s_live_containers_paginated(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<PaginatedResponse<InfoContainerEntity>> {
    const DEFAULT_LIMIT: usize = 50;

    let client = build_kube_client().await?;
    let pods = fetch_pods(&client).await?;

    let mut containers = Vec::new();

    for pod in pods {
        let spec = match pod.spec.as_ref() {
            Some(spec) => spec,
            None => continue,
        };

        for c in &spec.containers {
            let mut mapped = map_container_from_pod(&pod, &c.name)?;
            mapped.last_updated_info_at = Some(Utc::now());
            containers.push(mapped);
        }
    }

    let total = containers.len();
    let offset = offset.unwrap_or(0).min(total);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let end = (offset + limit).min(total);

    let items = containers
        .into_iter()
        .skip(offset)
        .take(end.saturating_sub(offset))
        .collect();

    Ok(PaginatedResponse {
        items,
        total,
        limit: end.saturating_sub(offset),
        offset,
    })
}

pub async fn get_k8s_live_container(id: String) -> Result<InfoContainerEntity> {
    let client = build_kube_client().await?;
    let pods = fetch_pods(&client).await?;

    for pod in pods {
        let spec = match pod.spec.as_ref() {
            Some(spec) => spec,
            None => continue,
        };

        for c in &spec.containers {
            let mapped = map_container_from_pod(&pod, &c.name)?;
            if mapped.container_id.as_deref() == Some(&id) {
                return Ok(mapped);
            }
        }
    }

    Err(anyhow!("Container '{}' not found", id))
}
