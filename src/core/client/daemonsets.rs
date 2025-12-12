use anyhow::Result;
use kube::{Api, Client};
use kube::api::ListParams;
use tracing::debug;

use crate::core::client::kube_resources::DaemonSet;

/// Fetch all daemonsets in the cluster
pub async fn fetch_daemonsets(client: &Client) -> Result<Vec<DaemonSet>> {
    let daemonsets: Api<DaemonSet> = Api::all(client.clone());
    let ds_list = daemonsets.list(&ListParams::default()).await?;

    debug!("Discovered {} daemonset(s)", ds_list.items.len());
    Ok(ds_list.items)
}

/// Fetch daemonsets in a specific namespace
pub async fn fetch_daemonsets_by_namespace(
    client: &Client,
    namespace: &str,
) -> Result<Vec<DaemonSet>> {
    let daemonsets: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
    let ds_list = daemonsets.list(&ListParams::default()).await?;

    debug!(
        "Discovered {} daemonset(s) in namespace '{}'",
        ds_list.items.len(),
        namespace
    );
    Ok(ds_list.items)
}

/// Fetch a single daemonset by name and namespace
pub async fn fetch_daemonset_by_name_and_namespace(
    client: &Client,
    namespace: &str,
    name: &str,
) -> Result<DaemonSet> {
    let daemonsets: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
    let ds = daemonsets.get(name).await?;

    debug!("Fetched daemonset: {}/{}", namespace, name);
    Ok(ds)
}

/// Fetch daemonsets filtered by label selector
pub async fn fetch_daemonsets_by_label(
    client: &Client,
    label_selector: &str,
) -> Result<Vec<DaemonSet>> {
    let daemonsets: Api<DaemonSet> = Api::all(client.clone());
    let lp = ListParams::default().labels(label_selector);
    let ds_list = daemonsets.list(&lp).await?;

    debug!(
        "Found {} daemonset(s) with label '{}'",
        ds_list.items.len(),
        label_selector
    );
    Ok(ds_list.items)
}
