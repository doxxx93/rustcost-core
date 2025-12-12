use anyhow::Result;
use kube::{Api, Client};
use kube::api::ListParams;
use tracing::debug;

use crate::core::client::kube_resources::StatefulSet;

/// Fetch all statefulsets in the cluster
pub async fn fetch_statefulsets(client: &Client) -> Result<Vec<StatefulSet>> {
    let statefulsets: Api<StatefulSet> = Api::all(client.clone());
    let statefulset_list = statefulsets.list(&ListParams::default()).await?;

    debug!(
        "Discovered {} statefulset(s)",
        statefulset_list.items.len()
    );
    Ok(statefulset_list.items)
}

/// Fetch statefulsets in a specific namespace
pub async fn fetch_statefulsets_by_namespace(
    client: &Client,
    namespace: &str,
) -> Result<Vec<StatefulSet>> {
    let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
    let statefulset_list = statefulsets.list(&ListParams::default()).await?;

    debug!(
        "Discovered {} statefulset(s) in namespace '{}'",
        statefulset_list.items.len(),
        namespace
    );
    Ok(statefulset_list.items)
}

/// Fetch a single statefulset by name and namespace
pub async fn fetch_statefulset_by_name_and_namespace(
    client: &Client,
    namespace: &str,
    statefulset_name: &str,
) -> Result<StatefulSet> {
    let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
    let statefulset = statefulsets.get(statefulset_name).await?;

    debug!("Fetched statefulset: {}/{}", namespace, statefulset_name);
    Ok(statefulset)
}

/// Fetch statefulsets filtered by label selector
pub async fn fetch_statefulsets_by_label(
    client: &Client,
    label_selector: &str,
) -> Result<Vec<StatefulSet>> {
    let statefulsets: Api<StatefulSet> = Api::all(client.clone());
    let lp = ListParams::default().labels(label_selector);
    let statefulset_list = statefulsets.list(&lp).await?;

    debug!(
        "Found {} statefulset(s) with label '{}'",
        statefulset_list.items.len(),
        label_selector
    );
    Ok(statefulset_list.items)
}
