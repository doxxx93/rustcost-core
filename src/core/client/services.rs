use anyhow::Result;
use kube::{Api, Client};
use kube::api::ListParams;
use tracing::debug;

use crate::core::client::kube_resources::Service;

/// Fetch all services in the cluster
pub async fn fetch_services(client: &Client) -> Result<Vec<Service>> {
    let services: Api<Service> = Api::all(client.clone());
    let svc_list = services.list(&ListParams::default()).await?;

    debug!("Discovered {} service(s)", svc_list.items.len());
    Ok(svc_list.items)
}

/// Fetch services in a specific namespace
pub async fn fetch_services_by_namespace(
    client: &Client,
    namespace: &str,
) -> Result<Vec<Service>> {
    let services: Api<Service> = Api::namespaced(client.clone(), namespace);
    let svc_list = services.list(&ListParams::default()).await?;

    debug!(
        "Discovered {} service(s) in namespace '{}'",
        svc_list.items.len(),
        namespace
    );
    Ok(svc_list.items)
}

/// Fetch a single service by name and namespace
pub async fn fetch_service_by_name_and_namespace(
    client: &Client,
    namespace: &str,
    name: &str,
) -> Result<Service> {
    let services: Api<Service> = Api::namespaced(client.clone(), namespace);
    let svc = services.get(name).await?;

    debug!("Fetched service: {}/{}", namespace, name);
    Ok(svc)
}

/// Fetch services filtered by label selector
pub async fn fetch_services_by_label(
    client: &Client,
    label_selector: &str,
) -> Result<Vec<Service>> {
    let services: Api<Service> = Api::all(client.clone());
    let lp = ListParams::default().labels(label_selector);
    let svc_list = services.list(&lp).await?;

    debug!(
        "Found {} service(s) with label '{}'",
        svc_list.items.len(),
        label_selector
    );
    Ok(svc_list.items)
}
