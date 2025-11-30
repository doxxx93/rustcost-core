use anyhow::{Context, Result};
use kube::{
    api::{Api, ListParams},
    Client,
};
use tracing::{error, info};
use crate::core::state::runtime::k8s::k8s_runtime_state::RuntimePod;
use crate::core::state::runtime::k8s::k8s_runtime_state_manager::K8sRuntimeStateManager;

/// Fetch all Kubernetes objects your runtime state cares about, and update your
/// `K8sRuntimeState` in memory.
///
/// This is a full discovery cycle.
pub async fn refresh_k8s_object_info<R>(
    manager: &K8sRuntimeStateManager<R>,
) -> Result<()>
where
    R: crate::core::state::runtime::k8s::k8s_runtime_state_repository_trait::K8sRuntimeStateRepositoryTrait,
{
    let client = crate::core::client::kube_client::build_kube_client()
        .await
        .context("failed to create kube client")?;

    info!("Refreshing Kubernetes runtime state...");

    // ---------------------------
    // 1. LOAD NODES
    // ---------------------------
    let nodes_api: Api<k8s_openapi::api::core::v1::Node> = Api::all(client.clone());
    let nodes = nodes_api
        .list(&ListParams::default())
        .await
        .context("failed to list nodes")?;

    let node_names: Vec<String> = nodes
        .items
        .into_iter()
        .filter_map(|n| n.metadata.name)
        .collect();

    // ---------------------------
    // 2. LOAD NAMESPACES
    // ---------------------------
    let ns_api: Api<k8s_openapi::api::core::v1::Namespace> = Api::all(client.clone());
    let namespaces = ns_api
        .list(&ListParams::default())
        .await
        .context("failed to list namespaces")?;

    let namespace_names: Vec<String> = namespaces
        .items
        .into_iter()
        .filter_map(|ns| ns.metadata.name)
        .collect();

    // ---------------------------
    // 3. LOAD DEPLOYMENTS
    // ---------------------------
    let deploy_api: Api<k8s_openapi::api::apps::v1::Deployment> = Api::all(client.clone());
    let deployments = deploy_api
        .list(&ListParams::default())
        .await
        .context("failed to list deployments")?;

    let deployment_names: Vec<String> = deployments
        .items
        .into_iter()
        .filter_map(|d| d.metadata.name)
        .collect();

    // ---------------------------
    // 4. LOAD PODS
    // ---------------------------
    let pod_api: Api<k8s_openapi::api::core::v1::Pod> = Api::all(client.clone());
    let pods = pod_api
        .list(&ListParams::default())
        .await
        .context("failed to list pods")?;

    let mut runtime_pods = Vec::<RuntimePod>::new();

    for pod in pods.items {
        let metadata = pod.metadata;
        let spec = pod.spec.clone();
        let pod_name = metadata.name.clone().unwrap_or_default();
        let namespace = metadata.namespace.clone().unwrap_or_default();
        let uid = metadata.uid.clone().unwrap_or_else(|| format!("{}-no-uid", pod_name));

        // Node assignment (may be empty for pending pods)
        let node = spec
            .as_ref()
            .and_then(|s| s.node_name.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Deployment inference from labels
        let deployment = metadata
            .labels
            .as_ref()
            .and_then(|lbl| lbl.get("app.kubernetes.io/name").cloned()) // common label
            .or_else(|| {
                metadata
                    .owner_references
                    .as_ref()
                    .and_then(|owners| {
                        owners
                            .iter()
                            .find(|o| o.kind == "ReplicaSet")
                            .and_then(|owner| {
                                // drop last "-<hash>" if present
                                let rs = owner.name.clone();
                                rs.rsplit_once('-').map(|(base, _)| base.to_string())
                            })
                    })
            });

        // Container names
        let containers = pod
            .spec
            .clone()
            .map(|s| {
                s.containers
                    .into_iter()
                    .map(|c| c.name)
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        runtime_pods.push(RuntimePod {
            uid,
            name: pod_name,
            namespace,
            deployment,
            node,
            containers,
        });
    }

    // ---------------------------
    // 5. UPDATE RUNTIME STATE
    // ---------------------------
    info!(
        "K8s discovery complete: {} nodes, {} namespaces, {} deployments, {} pods",
        node_names.len(),
        namespace_names.len(),
        deployment_names.len(),
        runtime_pods.len(),
    );

    if let Err(e) = manager
        .update_discovery(
            node_names,
            namespace_names,
            deployment_names,
            runtime_pods,
        )
        .await
    {
        error!("failed to update discovery state: {e}");
        manager
            .mark_error(format!("Failed to update discovery state: {e}"))
            .await;
        return Err(e);
    }

    Ok(())
}
