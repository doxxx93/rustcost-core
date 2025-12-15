use anyhow::Result;
use k8s_openapi::api::core::v1::Pod;

use crate::api::dto::paginated_response::PaginatedResponse;
use crate::core::client::kube_client::build_kube_client;
use crate::core::client::pods::{fetch_pod_by_uid, fetch_pods};

pub async fn get_k8s_live_pods_paginated(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<PaginatedResponse<Pod>> {
    const DEFAULT_LIMIT: usize = 50;

    let client = build_kube_client().await?;
    let pods = fetch_pods(&client).await?;
    let total = pods.len();

    let offset = offset.unwrap_or(0).min(total);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let end = (offset + limit).min(total);

    let items = pods
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

pub async fn get_k8s_live_pod(pod_uid: String) -> Result<Pod> {
    let client = build_kube_client().await?;
    fetch_pod_by_uid(&client, &pod_uid).await
}
