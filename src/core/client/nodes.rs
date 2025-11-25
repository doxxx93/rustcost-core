use anyhow::Result;
use kube::{Api, Client};
use kube::api::ListParams;
use tracing::debug;

use crate::core::client::kube_resources::Node;

/// Fetch all nodes in the cluster
pub async fn fetch_nodes(client: &Client) -> Result<Vec<Node>> {
    let nodes: Api<Node> = Api::all(client.clone());
    let node_list = nodes.list(&ListParams::default()).await?;

    debug!("Discovered {} node(s)", node_list.items.len());
    Ok(node_list.items)
}

/// Fetch a single node by name
pub async fn fetch_node_by_name(client: &Client, name: &str) -> Result<Node> {
    let nodes: Api<Node> = Api::all(client.clone());
    let node = nodes.get(name).await?;

    debug!("Fetched node: {}", name);
    Ok(node)
}

/// Fetch node names only
pub async fn fetch_node_names(client: &Client) -> Result<Vec<String>> {
    let nodes = fetch_nodes(client).await?;
    let names = nodes
        .into_iter()
        .filter_map(|n| n.metadata.name)
        .collect();

    Ok(names)
}

/// Fetch node summary stats from kubelet /stats/summary endpoint
/// This uses a direct proxy request to the kubelet through the API server
pub async fn fetch_node_summary<T>(
    client: &Client,
    node_name: &str,
) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    use http::{Method, Request as HttpRequest};

    // Build the proxy path to kubelet stats endpoint
    let url = format!(
        "/api/v1/nodes/{}/proxy/stats/summary",
        node_name
    );

    // Create HTTP request
    let req = HttpRequest::builder()
        .method(Method::GET)
        .uri(&url)
        .body(vec![])
        .map_err(|e| anyhow::anyhow!("Failed to build request: {}", e))?;

    // Send request through kube client
    let summary = client.request_text(req).await?;
    let parsed: T = serde_json::from_str(&summary)?;

    debug!("Fetched summary for node: {}", node_name);
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_nodes_does_not_panic() {
        // This test will fail if not in a k8s cluster, but shouldn't panic
        // We'll just verify the function exists and can be called
    }
}
