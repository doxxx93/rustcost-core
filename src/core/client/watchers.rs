use anyhow::Result;
use futures::StreamExt;
use kube::{Api, Client};
use kube::runtime::{watcher, WatchStreamExt};
use tracing::{debug, error, info};

use crate::core::client::kube_resources::{Node, Pod, Deployment};

/// Watch for Node changes in real-time
/// This function streams Node events (Added/Modified/Deleted)
pub async fn watch_nodes<F>(client: &Client, mut handler: F) -> Result<()>
where
    F: FnMut(Node) -> Result<()>,
{
    let api: Api<Node> = Api::all(client.clone());
    let watcher_config = watcher::Config::default();

    info!("Starting Node watcher...");

    let mut stream = watcher(api, watcher_config)
        .applied_objects()
        .boxed();

    while let Some(result) = stream.next().await {
        match result {
            Ok(node) => {
                debug!("Node event: {}", node.metadata.name.as_deref().unwrap_or("unknown"));

                if let Err(e) = handler(node) {
                    error!("Error handling node event: {:?}", e);
                }
            }
            Err(e) => {
                error!("Node watcher error: {:?}", e);
                // Watcher will auto-reconnect on most errors
            }
        }
    }

    Ok(())
}

/// Watch for Pod changes in real-time
pub async fn watch_pods<F>(client: &Client, mut handler: F) -> Result<()>
where
    F: FnMut(Pod) -> Result<()>,
{
    let api: Api<Pod> = Api::all(client.clone());
    let watcher_config = watcher::Config::default();

    info!("Starting Pod watcher...");

    let mut stream = watcher(api, watcher_config)
        .applied_objects()
        .boxed();

    while let Some(result) = stream.next().await {
        match result {
            Ok(pod) => {
                let pod_name = pod.metadata.name.as_deref().unwrap_or("unknown");
                let namespace = pod.metadata.namespace.as_deref().unwrap_or("default");
                debug!("Pod event: {}/{}", namespace, pod_name);

                if let Err(e) = handler(pod) {
                    error!("Error handling pod event: {:?}", e);
                }
            }
            Err(e) => {
                error!("Pod watcher error: {:?}", e);
            }
        }
    }

    Ok(())
}

/// Watch for Deployment changes in real-time
pub async fn watch_deployments<F>(client: &Client, mut handler: F) -> Result<()>
where
    F: FnMut(Deployment) -> Result<()>,
{
    let api: Api<Deployment> = Api::all(client.clone());
    let watcher_config = watcher::Config::default();

    info!("Starting Deployment watcher...");

    let mut stream = watcher(api, watcher_config)
        .applied_objects()
        .boxed();

    while let Some(result) = stream.next().await {
        match result {
            Ok(deployment) => {
                let name = deployment.metadata.name.as_deref().unwrap_or("unknown");
                let namespace = deployment.metadata.namespace.as_deref().unwrap_or("default");
                debug!("Deployment event: {}/{}", namespace, name);

                if let Err(e) = handler(deployment) {
                    error!("Error handling deployment event: {:?}", e);
                }
            }
            Err(e) => {
                error!("Deployment watcher error: {:?}", e);
            }
        }
    }

    Ok(())
}

/// Example: Watch pods in a specific namespace
pub async fn watch_pods_in_namespace<F>(
    client: &Client,
    namespace: &str,
    mut handler: F,
) -> Result<()>
where
    F: FnMut(Pod) -> Result<()>,
{
    let api: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let watcher_config = watcher::Config::default();

    info!("Starting Pod watcher for namespace '{}'...", namespace);

    let mut stream = watcher(api, watcher_config)
        .applied_objects()
        .boxed();

    while let Some(result) = stream.next().await {
        match result {
            Ok(pod) => {
                let pod_name = pod.metadata.name.as_deref().unwrap_or("unknown");
                debug!("Pod event in {}: {}", namespace, pod_name);

                if let Err(e) = handler(pod) {
                    error!("Error handling pod event: {:?}", e);
                }
            }
            Err(e) => {
                error!("Pod watcher error: {:?}", e);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_watchers_compile() {
        // This test just verifies that watchers compile correctly
    }
}
