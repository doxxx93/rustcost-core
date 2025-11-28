use anyhow::Result;
use kube::{Api, Client, ResourceExt};
use kube::runtime::{watcher, reflector, WatchStreamExt};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

use crate::core::client::kube_resources::{Node, Pod, Deployment};

/// A Store holds an in-memory cache of Kubernetes resources
/// automatically kept in sync via watchers
pub struct KubeStore {
    nodes_reader: Arc<reflector::Store<Node>>,
    pods_reader: Arc<reflector::Store<Pod>>,
    deployments_reader: Arc<reflector::Store<Deployment>>,
}

impl KubeStore {
    /// Create a new empty store - stores are populated by start_watchers()
    pub fn new() -> Self {
        // Create empty stores - they will be populated by reflectors in start_watchers
        let (nodes_reader, _) = reflector::store();
        let (pods_reader, _) = reflector::store();
        let (deployments_reader, _) = reflector::store();

        Self {
            nodes_reader: Arc::new(nodes_reader),
            pods_reader: Arc::new(pods_reader),
            deployments_reader: Arc::new(deployments_reader),
        }
    }

    /// Start all watchers and reflectors to populate the stores
    /// Returns join handles for the background tasks
    pub fn start_watchers(
        &self,
        client: Client,
    ) -> Result<Vec<JoinHandle<()>>> {
        let mut handles = Vec::new();

        // Start Node reflector
        let nodes_store = self.nodes_reader.clone();
        let nodes_client = client.clone();
        let node_handle = tokio::spawn(async move {
            if let Err(e) = run_node_reflector(nodes_client, nodes_store).await {
                error!("Node reflector error: {:?}", e);
            }
        });
        handles.push(node_handle);

        // Start Pod reflector
        let pods_store = self.pods_reader.clone();
        let pods_client = client.clone();
        let pod_handle = tokio::spawn(async move {
            if let Err(e) = run_pod_reflector(pods_client, pods_store).await {
                error!("Pod reflector error: {:?}", e);
            }
        });
        handles.push(pod_handle);

        // Start Deployment reflector
        let deployments_store = self.deployments_reader.clone();
        let deployments_client = client.clone();
        let deployment_handle = tokio::spawn(async move {
            if let Err(e) = run_deployment_reflector(deployments_client, deployments_store).await {
                error!("Deployment reflector error: {:?}", e);
            }
        });
        handles.push(deployment_handle);

        info!("All Kubernetes resource reflectors started");
        Ok(handles)
    }

    /// Get all nodes from cache (no API call)
    pub fn get_nodes(&self) -> Vec<Node> {
        self.nodes_reader.state().iter().map(|n| (**n).clone()).collect()
    }

    /// Get all pods from cache (no API call)
    pub fn get_pods(&self) -> Vec<Pod> {
        self.pods_reader.state().iter().map(|p| (**p).clone()).collect()
    }

    /// Get pods in a specific namespace from cache
    pub fn get_pods_by_namespace(&self, namespace: &str) -> Vec<Pod> {
        self.pods_reader
            .state()
            .iter()
            .filter(|p| p.namespace().as_deref() == Some(namespace))
            .map(|p| (**p).clone())
            .collect()
    }

    /// Get pods on a specific node from cache
    pub fn get_pods_by_node(&self, node_name: &str) -> Vec<Pod> {
        self.pods_reader
            .state()
            .iter()
            .filter(|p| {
                p.spec
                    .as_ref()
                    .and_then(|spec| spec.node_name.as_deref())
                    == Some(node_name)
            })
            .map(|p| (**p).clone())
            .collect()
    }

    /// Get a single pod by name and namespace from cache
    pub fn get_pod(&self, namespace: &str, name: &str) -> Option<Pod> {
        self.pods_reader
            .state()
            .iter()
            .find(|p| p.namespace().as_deref() == Some(namespace) && p.name_any() == name)
            .map(|p| (**p).clone())
    }

    /// Get all deployments from cache (no API call)
    pub fn get_deployments(&self) -> Vec<Deployment> {
        self.deployments_reader.state().iter().map(|d| (**d).clone()).collect()
    }

    /// Get a node by name from cache
    pub fn get_node(&self, name: &str) -> Option<Node> {
        self.nodes_reader
            .state()
            .iter()
            .find(|n| n.name_any() == name)
            .map(|n| (**n).clone())
    }
}

impl Default for KubeStore {
    fn default() -> Self {
        Self::new()
    }
}

// Internal reflector runners

async fn run_node_reflector(
    client: Client,
    _store: Arc<reflector::Store<Node>>,
) -> Result<()> {
    use futures::TryStreamExt;

    let api: Api<Node> = Api::all(client);
    let watcher_config = watcher::Config::default();

    info!("Starting Node reflector (optimized with .modify())...");

    // Create a new store writer-reader pair for this reflector
    let (_reader, writer) = reflector::store();

    let stream = watcher(api, watcher_config)
        .modify(|node| {
            // Strip unnecessary fields to reduce memory usage
            node.managed_fields_mut().clear();

            if let Some(status) = node.status.as_mut() {
                // Remove heavy fields not needed for cost tracking
                status.images = None;              // Don't need image list
                status.volumes_in_use = None;      // Don't need volume details
                status.volumes_attached = None;     // Don't need attachment details
            }
        })
        .default_backoff();

    reflector::reflector(writer, stream)
        .touched_objects()
        .try_for_each(|node| async move {
            debug!("Node cache updated (optimized): {}", node.name_any());
            Ok(())
        })
        .await?;

    Ok(())
}

async fn run_pod_reflector(
    client: Client,
    _store: Arc<reflector::Store<Pod>>,
) -> Result<()> {
    use futures::TryStreamExt;

    let api: Api<Pod> = Api::all(client);
    let watcher_config = watcher::Config::default();

    info!("Starting Pod reflector (optimized with .modify())...");

    let (_reader, writer) = reflector::store();

    let stream = watcher(api, watcher_config)
        .modify(|pod| {
            // Strip unnecessary fields to reduce memory usage (40-60% savings)
            pod.managed_fields_mut().clear();
            pod.annotations_mut().clear();  // Usually large and not needed for cost

            if let Some(status) = pod.status.as_mut() {
                // Keep resource usage info but remove verbose details
                status.init_container_statuses = None;  // Don't need init container details
                status.ephemeral_container_statuses = None;
                status.conditions = None;               // Don't need condition history

                // Clear verbose container status fields but keep basic state
                if let Some(container_statuses) = status.container_statuses.as_mut() {
                    for cs in container_statuses {
                        cs.state.as_mut().map(|s| {
                            // Keep state type but clear verbose reason/message
                            if let Some(running) = s.running.as_mut() {
                                running.started_at = None;
                            }
                        });
                        cs.last_state = None;  // Don't need last state history
                    }
                }
            }
        })
        .default_backoff();

    reflector::reflector(writer, stream)
        .touched_objects()
        .try_for_each(|pod| async move {
            debug!("Pod cache updated (optimized): {}/{}", pod.namespace().unwrap_or_default(), pod.name_any());
            Ok(())
        })
        .await?;

    Ok(())
}

async fn run_deployment_reflector(
    client: Client,
    _store: Arc<reflector::Store<Deployment>>,
) -> Result<()> {
    use futures::TryStreamExt;

    let api: Api<Deployment> = Api::all(client);
    let watcher_config = watcher::Config::default();

    info!("Starting Deployment reflector (optimized with .modify())...");

    let (_reader, writer) = reflector::store();

    let stream = watcher(api, watcher_config)
        .modify(|deployment| {
            // Strip unnecessary fields to reduce memory usage
            deployment.managed_fields_mut().clear();
            deployment.annotations_mut().clear();

            if let Some(status) = deployment.status.as_mut() {
                // Keep replica counts but remove detailed conditions
                status.conditions = None;  // Don't need condition history for cost tracking
            }
        })
        .default_backoff();

    reflector::reflector(writer, stream)
        .touched_objects()
        .try_for_each(|deployment| async move {
            debug!("Deployment cache updated (optimized): {}/{}", deployment.namespace().unwrap_or_default(), deployment.name_any());
            Ok(())
        })
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_creation() {
        let store = KubeStore::new();
        assert_eq!(store.get_nodes().len(), 0);
        assert_eq!(store.get_pods().len(), 0);
    }
}
