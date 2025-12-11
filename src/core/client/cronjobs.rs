use anyhow::Result;
use kube::{Api, Client};
use kube::api::ListParams;
use tracing::debug;

use crate::core::client::kube_resources::CronJob;

/// Fetch all cronjobs in the cluster
pub async fn fetch_cronjobs(client: &Client) -> Result<Vec<CronJob>> {
    let cronjobs: Api<CronJob> = Api::all(client.clone());
    let cj_list = cronjobs.list(&ListParams::default()).await?;

    debug!("Discovered {} cronjob(s)", cj_list.items.len());
    Ok(cj_list.items)
}

/// Fetch cronjobs in a specific namespace
pub async fn fetch_cronjobs_by_namespace(
    client: &Client,
    namespace: &str,
) -> Result<Vec<CronJob>> {
    let cronjobs: Api<CronJob> = Api::namespaced(client.clone(), namespace);
    let cj_list = cronjobs.list(&ListParams::default()).await?;

    debug!(
        "Discovered {} cronjob(s) in namespace '{}'",
        cj_list.items.len(),
        namespace
    );
    Ok(cj_list.items)
}

/// Fetch a single cronjob by name and namespace
pub async fn fetch_cronjob_by_name_and_namespace(
    client: &Client,
    namespace: &str,
    name: &str,
) -> Result<CronJob> {
    let cronjobs: Api<CronJob> = Api::namespaced(client.clone(), namespace);
    let cj = cronjobs.get(name).await?;

    debug!("Fetched cronjob: {}/{}", namespace, name);
    Ok(cj)
}

/// Fetch cronjobs filtered by label selector
pub async fn fetch_cronjobs_by_label(
    client: &Client,
    label_selector: &str,
) -> Result<Vec<CronJob>> {
    let cronjobs: Api<CronJob> = Api::all(client.clone());
    let lp = ListParams::default().labels(label_selector);
    let cj_list = cronjobs.list(&lp).await?;

    debug!(
        "Found {} cronjob(s) with label '{}'",
        cj_list.items.len(),
        label_selector
    );
    Ok(cj_list.items)
}
