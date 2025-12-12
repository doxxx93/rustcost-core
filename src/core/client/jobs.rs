use anyhow::Result;
use kube::{Api, Client};
use kube::api::ListParams;
use tracing::debug;

use crate::core::client::kube_resources::Job;

/// Fetch all jobs in the cluster
pub async fn fetch_jobs(client: &Client) -> Result<Vec<Job>> {
    let jobs: Api<Job> = Api::all(client.clone());
    let job_list = jobs.list(&ListParams::default()).await?;

    debug!("Discovered {} job(s)", job_list.items.len());
    Ok(job_list.items)
}

/// Fetch jobs in a specific namespace
pub async fn fetch_jobs_by_namespace(client: &Client, namespace: &str) -> Result<Vec<Job>> {
    let jobs: Api<Job> = Api::namespaced(client.clone(), namespace);
    let job_list = jobs.list(&ListParams::default()).await?;

    debug!(
        "Discovered {} job(s) in namespace '{}'",
        job_list.items.len(),
        namespace
    );
    Ok(job_list.items)
}

/// Fetch a single job by name and namespace
pub async fn fetch_job_by_name_and_namespace(
    client: &Client,
    namespace: &str,
    name: &str,
) -> Result<Job> {
    let jobs: Api<Job> = Api::namespaced(client.clone(), namespace);
    let job = jobs.get(name).await?;

    debug!("Fetched job: {}/{}", namespace, name);
    Ok(job)
}

/// Fetch jobs filtered by label selector
pub async fn fetch_jobs_by_label(client: &Client, label_selector: &str) -> Result<Vec<Job>> {
    let jobs: Api<Job> = Api::all(client.clone());
    let lp = ListParams::default().labels(label_selector);
    let job_list = jobs.list(&lp).await?;

    debug!(
        "Found {} job(s) with label '{}'",
        job_list.items.len(),
        label_selector
    );
    Ok(job_list.items)
}
