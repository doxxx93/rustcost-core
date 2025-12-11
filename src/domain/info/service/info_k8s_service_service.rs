use anyhow::Result;
use k8s_openapi::api::core::v1::Service;

use crate::api::dto::paginated_response::PaginatedResponse;
use crate::core::client::k8s::client_k8s_service;
use crate::core::client::k8s::util::{build_client, read_token};

pub async fn get_k8s_services() -> Result<PaginatedResponse<Service>> {
    get_k8s_services_paginated(None, None).await
}

pub async fn get_k8s_services_paginated(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<PaginatedResponse<Service>> {
    const DEFAULT_LIMIT: usize = 50;

    let token = read_token()?;
    let client = build_client()?;

    let services = client_k8s_service::fetch_services(&token, &client).await?;
    let total = services.items.len();

    let offset = offset.unwrap_or(0).min(total);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let end = (offset + limit).min(total);

    let items = services
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

pub async fn get_k8s_service(namespace: String, name: String) -> Result<Service> {
    let token = read_token()?;
    let client = build_client()?;

    client_k8s_service::fetch_service_by_name_and_namespace(&token, &client, &namespace, &name)
        .await
}
