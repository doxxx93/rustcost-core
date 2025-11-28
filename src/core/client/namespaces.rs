use anyhow::Result;
use kube::{Api, Client};
use kube::api::ListParams;
use tracing::debug;

use crate::core::client::kube_resources::Namespace;

/// Fetch all namespaces in the cluster
pub async fn fetch_namespaces(client: &Client) -> Result<Vec<Namespace>> {
    let namespaces: Api<Namespace> = Api::all(client.clone());
    let namespace_list = namespaces.list(&ListParams::default()).await?;

    debug!("Discovered {} namespace(s)", namespace_list.items.len());
    Ok(namespace_list.items)
}

/// Fetch a single namespace by name
pub async fn fetch_namespace_by_name(client: &Client, name: &str) -> Result<Namespace> {
    let namespaces: Api<Namespace> = Api::all(client.clone());
    let namespace = namespaces.get(name).await?;

    debug!("Fetched namespace: {}", name);
    Ok(namespace)
}

/// Fetch namespace names only
pub async fn fetch_namespace_names(client: &Client) -> Result<Vec<String>> {
    let namespaces = fetch_namespaces(client).await?;
    let names = namespaces
        .into_iter()
        .filter_map(|n| n.metadata.name)
        .collect();

    Ok(names)
}
