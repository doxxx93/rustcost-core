use anyhow::Result;
use k8s_openapi::api::core::v1::Node;

use crate::api::dto::paginated_response::PaginatedResponse;
use crate::core::client::kube_client::build_kube_client;
use crate::core::client::nodes::{fetch_node_by_name, fetch_nodes};

pub async fn get_k8s_live_nodes_paginated(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<PaginatedResponse<Node>> {
    const DEFAULT_LIMIT: usize = 50;

    let client = build_kube_client().await?;
    let nodes = fetch_nodes(&client).await?;
    let total = nodes.len();

    let offset = offset.unwrap_or(0).min(total);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let end = (offset + limit).min(total);

    let items = nodes
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

pub async fn get_k8s_live_node(node_name: String) -> Result<Node> {
    let client = build_kube_client().await?;
    fetch_node_by_name(&client, &node_name).await
}
