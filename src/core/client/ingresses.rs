use anyhow::Result;
use kube::{Api, Client};
use kube::api::ListParams;
use tracing::debug;

use crate::core::client::kube_resources::Ingress;

/// Fetch all ingresses in the cluster
pub async fn fetch_ingresses(client: &Client) -> Result<Vec<Ingress>> {
    let ingresses: Api<Ingress> = Api::all(client.clone());
    let ingress_list = ingresses.list(&ListParams::default()).await?;

    debug!("Discovered {} ingress(es)", ingress_list.items.len());
    Ok(ingress_list.items)
}

/// Fetch ingresses in a specific namespace
pub async fn fetch_ingresses_by_namespace(
    client: &Client,
    namespace: &str,
) -> Result<Vec<Ingress>> {
    let ingresses: Api<Ingress> = Api::namespaced(client.clone(), namespace);
    let ingress_list = ingresses.list(&ListParams::default()).await?;

    debug!(
        "Discovered {} ingress(es) in namespace '{}'",
        ingress_list.items.len(),
        namespace
    );
    Ok(ingress_list.items)
}

/// Fetch a single ingress by name and namespace
pub async fn fetch_ingress_by_name_and_namespace(
    client: &Client,
    namespace: &str,
    name: &str,
) -> Result<Ingress> {
    let ingresses: Api<Ingress> = Api::namespaced(client.clone(), namespace);
    let ingress = ingresses.get(name).await?;

    debug!("Fetched ingress: {}/{}", namespace, name);
    Ok(ingress)
}

/// Fetch ingresses filtered by label selector
pub async fn fetch_ingresses_by_label(
    client: &Client,
    label_selector: &str,
) -> Result<Vec<Ingress>> {
    let ingresses: Api<Ingress> = Api::all(client.clone());
    let lp = ListParams::default().labels(label_selector);
    let ingress_list = ingresses.list(&lp).await?;

    debug!(
        "Found {} ingress(es) with label '{}'",
        ingress_list.items.len(),
        label_selector
    );
    Ok(ingress_list.items)
}
