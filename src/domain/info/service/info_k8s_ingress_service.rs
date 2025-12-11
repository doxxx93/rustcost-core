use anyhow::Result;
use k8s_openapi::api::networking::v1::Ingress;

use crate::api::dto::paginated_response::PaginatedResponse;
use crate::core::client::k8s::client_k8s_ingress;
use crate::core::client::k8s::util::{build_client, read_token};

pub async fn get_k8s_ingresses() -> Result<PaginatedResponse<Ingress>> {
    get_k8s_ingresses_paginated(None, None).await
}

pub async fn get_k8s_ingresses_paginated(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<PaginatedResponse<Ingress>> {
    const DEFAULT_LIMIT: usize = 50;

    let token = read_token()?;
    let client = build_client()?;

    let ingresses = client_k8s_ingress::fetch_ingresses(&token, &client).await?;
    let total = ingresses.items.len();

    let offset = offset.unwrap_or(0).min(total);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let end = (offset + limit).min(total);

    let items = ingresses
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

pub async fn get_k8s_ingress(namespace: String, name: String) -> Result<Ingress> {
    let token = read_token()?;
    let client = build_client()?;

    client_k8s_ingress::fetch_ingress_by_name_and_namespace(&token, &client, &namespace, &name)
        .await
}
