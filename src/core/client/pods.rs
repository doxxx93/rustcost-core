use anyhow::Result;
use kube::{Api, Client};
use kube::api::ListParams;
use tracing::debug;

use crate::core::client::kube_resources::Pod;

/// Fetch all pods in the cluster
pub async fn fetch_pods(client: &Client) -> Result<Vec<Pod>> {
    let pods: Api<Pod> = Api::all(client.clone());
    let pod_list = pods.list(&ListParams::default()).await?;

    debug!("Discovered {} pod(s)", pod_list.items.len());
    Ok(pod_list.items)
}

/// Fetch pods in a specific namespace
pub async fn fetch_pods_by_namespace(client: &Client, namespace: &str) -> Result<Vec<Pod>> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let pod_list = pods.list(&ListParams::default()).await?;

    debug!("Discovered {} pod(s) in namespace '{}'", pod_list.items.len(), namespace);
    Ok(pod_list.items)
}

/// Fetch a single pod by name and namespace
pub async fn fetch_pod_by_name_and_namespace(
    client: &Client,
    namespace: &str,
    pod_name: &str,
) -> Result<Pod> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let pod = pods.get(pod_name).await?;

    debug!("Fetched pod: {}/{}", namespace, pod_name);
    Ok(pod)
}

/// Fetch pods filtered by label selector (e.g. "app=myservice")
pub async fn fetch_pods_by_label(
    client: &Client,
    label_selector: &str,
) -> Result<Vec<Pod>> {
    let pods: Api<Pod> = Api::all(client.clone());
    let lp = ListParams::default().labels(label_selector);
    let pod_list = pods.list(&lp).await?;

    debug!("Found {} pod(s) with label '{}'", pod_list.items.len(), label_selector);
    Ok(pod_list.items)
}

/// Fetch pods scheduled on a specific node
pub async fn fetch_pods_by_node(client: &Client, node_name: &str) -> Result<Vec<Pod>> {
    let pods: Api<Pod> = Api::all(client.clone());
    let field_selector = format!("spec.nodeName={}", node_name);
    let lp = ListParams::default().fields(&field_selector);
    let pod_list = pods.list(&lp).await?;

    debug!("Found {} pod(s) on node '{}'", pod_list.items.len(), node_name);
    Ok(pod_list.items)
}

/// Fetch a pod by its UID
pub async fn fetch_pod_by_uid(client: &Client, pod_uid: &str) -> Result<Pod> {
    let pods: Api<Pod> = Api::all(client.clone());
    let field_selector = format!("metadata.uid={}", pod_uid);
    let lp = ListParams::default().fields(&field_selector);
    let pod_list = pods.list(&lp).await?;

    pod_list.items
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Pod with UID '{}' not found", pod_uid))
}

/// Fetch pod names only
pub async fn fetch_pod_names(client: &Client) -> Result<Vec<String>> {
    let pods = fetch_pods(client).await?;
    let names = pods
        .into_iter()
        .filter_map(|p| p.metadata.name)
        .collect();

    Ok(names)
}

/// Fetch pod names in a specific namespace
pub async fn fetch_pod_names_by_namespace(
    client: &Client,
    namespace: &str,
) -> Result<Vec<String>> {
    let pods = fetch_pods_by_namespace(client, namespace).await?;
    let names = pods
        .into_iter()
        .filter_map(|p| p.metadata.name)
        .collect();

    Ok(names)
}

/// Fetch pod names by label selector
pub async fn fetch_pod_names_by_label(
    client: &Client,
    label_selector: &str,
) -> Result<Vec<String>> {
    let pods = fetch_pods_by_label(client, label_selector).await?;
    let names = pods
        .into_iter()
        .filter_map(|p| p.metadata.name)
        .collect();

    Ok(names)
}

/// Fetch pod names on a specific node
pub async fn fetch_pod_names_by_node(client: &Client, node_name: &str) -> Result<Vec<String>> {
    let pods = fetch_pods_by_node(client, node_name).await?;
    let names = pods
        .into_iter()
        .filter_map(|p| p.metadata.name)
        .collect();

    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_pods_does_not_panic() {
        // Test exists to verify compilation
    }
}
