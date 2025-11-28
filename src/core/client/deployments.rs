use anyhow::Result;
use kube::{Api, Client};
use kube::api::ListParams;
use tracing::debug;

use crate::core::client::kube_resources::Deployment;

/// Fetch all deployments in the cluster
pub async fn fetch_deployments(client: &Client) -> Result<Vec<Deployment>> {
    let deployments: Api<Deployment> = Api::all(client.clone());
    let deployment_list = deployments.list(&ListParams::default()).await?;

    debug!("Discovered {} deployment(s)", deployment_list.items.len());
    Ok(deployment_list.items)
}

/// Fetch deployments in a specific namespace
pub async fn fetch_deployments_by_namespace(
    client: &Client,
    namespace: &str,
) -> Result<Vec<Deployment>> {
    let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);
    let deployment_list = deployments.list(&ListParams::default()).await?;

    debug!(
        "Discovered {} deployment(s) in namespace '{}'",
        deployment_list.items.len(),
        namespace
    );
    Ok(deployment_list.items)
}

/// Fetch a single deployment by name and namespace
pub async fn fetch_deployment_by_name_and_namespace(
    client: &Client,
    namespace: &str,
    deployment_name: &str,
) -> Result<Deployment> {
    let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);
    let deployment = deployments.get(deployment_name).await?;

    debug!("Fetched deployment: {}/{}", namespace, deployment_name);
    Ok(deployment)
}

/// Fetch deployments filtered by label selector
pub async fn fetch_deployments_by_label(
    client: &Client,
    label_selector: &str,
) -> Result<Vec<Deployment>> {
    let deployments: Api<Deployment> = Api::all(client.clone());
    let lp = ListParams::default().labels(label_selector);
    let deployment_list = deployments.list(&lp).await?;

    debug!(
        "Found {} deployment(s) with label '{}'",
        deployment_list.items.len(),
        label_selector
    );
    Ok(deployment_list.items)
}
