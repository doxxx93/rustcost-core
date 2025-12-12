use anyhow::Result;
use k8s_openapi::api::apps::v1::StatefulSet;

use crate::api::dto::paginated_response::PaginatedResponse;
use crate::core::client::k8s::client_k8s_statefulset;
use crate::core::client::k8s::util::{build_client, read_token};

pub async fn get_k8s_statefulsets() -> Result<PaginatedResponse<StatefulSet>> {
    get_k8s_statefulsets_paginated(None, None).await
}

pub async fn get_k8s_statefulsets_paginated(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<PaginatedResponse<StatefulSet>> {
    const DEFAULT_LIMIT: usize = 50;

    let token = read_token()?;
    let client = build_client()?;

    let statefulsets = client_k8s_statefulset::fetch_statefulsets(&token, &client).await?;
    let total = statefulsets.items.len();

    let offset = offset.unwrap_or(0).min(total);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let end = (offset + limit).min(total);

    let items = statefulsets
        .items
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

pub async fn get_k8s_statefulset(namespace: String, name: String) -> Result<StatefulSet> {
    let token = read_token()?;
    let client = build_client()?;

    client_k8s_statefulset::fetch_statefulset_by_name_and_namespace(
        &token,
        &client,
        &namespace,
        &name,
    )
    .await
}
