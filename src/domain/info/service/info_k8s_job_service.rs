use anyhow::Result;
use k8s_openapi::api::batch::v1::Job;

use crate::api::dto::paginated_response::PaginatedResponse;
use crate::core::client::k8s::client_k8s_job;
use crate::core::client::k8s::util::{build_client, read_token};

pub async fn get_k8s_jobs() -> Result<PaginatedResponse<Job>> {
    get_k8s_jobs_paginated(None, None).await
}

pub async fn get_k8s_jobs_paginated(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<PaginatedResponse<Job>> {
    const DEFAULT_LIMIT: usize = 50;

    let token = read_token()?;
    let client = build_client()?;

    let jobs = client_k8s_job::fetch_jobs(&token, &client).await?;
    let total = jobs.items.len();

    let offset = offset.unwrap_or(0).min(total);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let end = (offset + limit).min(total);

    let items = jobs
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

pub async fn get_k8s_job(namespace: String, name: String) -> Result<Job> {
    let token = read_token()?;
    let client = build_client()?;

    client_k8s_job::fetch_job_by_name_and_namespace(&token, &client, &namespace, &name).await
}
