use anyhow::Result;
use k8s_openapi::api::batch::v1::CronJob;

use crate::api::dto::paginated_response::PaginatedResponse;
use crate::core::client::k8s::client_k8s_cronjob;
use crate::core::client::k8s::util::{build_client, read_token};

pub async fn get_k8s_cronjobs() -> Result<PaginatedResponse<CronJob>> {
    get_k8s_cronjobs_paginated(None, None).await
}

pub async fn get_k8s_cronjobs_paginated(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<PaginatedResponse<CronJob>> {
    const DEFAULT_LIMIT: usize = 50;

    let token = read_token()?;
    let client = build_client()?;

    let cronjobs = client_k8s_cronjob::fetch_cronjobs(&token, &client).await?;
    let total = cronjobs.items.len();

    let offset = offset.unwrap_or(0).min(total);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let end = (offset + limit).min(total);

    let items = cronjobs
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

pub async fn get_k8s_cronjob(namespace: String, name: String) -> Result<CronJob> {
    let token = read_token()?;
    let client = build_client()?;

    client_k8s_cronjob::fetch_cronjob_by_name_and_namespace(&token, &client, &namespace, &name)
        .await
}
