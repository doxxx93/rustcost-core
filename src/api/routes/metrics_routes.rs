//! Metrics routes (e.g., /api/v1/metrics/*)

use axum::{routing::get, Router};

use crate::api::controller::metric::k8s::namespace::K8sNamespaceMetricsController;
use crate::api::controller::metric::k8s::node::K8sNodeMetricsController;
use crate::api::controller::metric::k8s::container::K8sContainerMetricsController;
use crate::api::controller::metric::k8s::deployment::K8sDeploymentMetricsController;
use crate::api::controller::metric::k8s::pod::K8sPodMetricsController;
use crate::api::controller::metric::k8s::cluster::K8sClusterMetricsController;
use crate::app_state::AppState;

/// Build the router for metrics endpoints under /api/v1/metrics
pub fn metrics_routes() -> Router<AppState> {
    Router::new()
        // Nodes
        .route("/nodes/raw", get(K8sNodeMetricsController::get_metric_k8s_nodes_raw))
        .route("/nodes/raw/summary", get(K8sNodeMetricsController::get_metric_k8s_nodes_raw_summary))
        .route("/nodes/raw/efficiency", get(K8sNodeMetricsController::get_metric_k8s_nodes_raw_efficiency))
        .route("/nodes/{node_name}/raw", get(K8sNodeMetricsController::get_metric_k8s_node_raw))
        .route("/nodes/{node_name}/raw/summary", get(K8sNodeMetricsController::get_metric_k8s_node_raw_summary))
        .route("/nodes/{node_name}/raw/efficiency", get(K8sNodeMetricsController::get_metric_k8s_node_raw_efficiency))
        .route("/nodes/cost", get(K8sNodeMetricsController::get_metric_k8s_nodes_cost))
        .route("/nodes/cost/summary", get(K8sNodeMetricsController::get_metric_k8s_nodes_cost_summary))
        .route("/nodes/cost/trend", get(K8sNodeMetricsController::get_metric_k8s_nodes_cost_trend))
        .route("/nodes/{node_name}/cost", get(K8sNodeMetricsController::get_metric_k8s_node_cost))
        .route("/nodes/{node_name}/cost/summary", get(K8sNodeMetricsController::get_metric_k8s_node_cost_summary))
        .route("/nodes/{node_name}/cost/trend", get(K8sNodeMetricsController::get_metric_k8s_node_cost_trend))

        // Pods
        .route("/pods/raw", get(K8sPodMetricsController::get_metric_k8s_pods_raw))
        .route("/pods/raw/summary", get(K8sPodMetricsController::get_metric_k8s_pods_raw_summary))
        .route("/pods/raw/efficiency", get(K8sPodMetricsController::get_metric_k8s_pods_raw_efficiency))
        .route("/pods/{pod_uid}/raw", get(K8sPodMetricsController::get_metric_k8s_pod_raw))
        .route("/pods/{pod_uid}/raw/summary", get(K8sPodMetricsController::get_metric_k8s_pod_raw_summary))
        .route("/pods/{pod_uid}/raw/efficiency", get(K8sPodMetricsController::get_metric_k8s_pod_raw_efficiency))
        .route("/pods/cost", get(K8sPodMetricsController::get_metric_k8s_pods_cost))
        .route("/pods/cost/summary", get(K8sPodMetricsController::get_metric_k8s_pods_cost_summary))
        .route("/pods/cost/trend", get(K8sPodMetricsController::get_metric_k8s_pods_cost_trend))
        .route("/pods/{pod_uid}/cost", get(K8sPodMetricsController::get_metric_k8s_pod_cost))
        .route("/pods/{pod_uid}/cost/summary", get(K8sPodMetricsController::get_metric_k8s_pod_cost_summary))
        .route("/pods/{pod_uid}/cost/trend", get(K8sPodMetricsController::get_metric_k8s_pod_cost_trend))

        // Containers
        .route("/containers/raw", get(K8sContainerMetricsController::get_metric_k8s_containers_raw))
        .route("/containers/raw/summary", get(K8sContainerMetricsController::get_metric_k8s_containers_raw_summary))
        .route("/containers/raw/efficiency", get(K8sContainerMetricsController::get_metric_k8s_containers_raw_efficiency))
        .route("/containers/{id}/raw", get(K8sContainerMetricsController::get_metric_k8s_container_raw))
        .route("/containers/{id}/raw/summary", get(K8sContainerMetricsController::get_metric_k8s_container_raw_summary))
        .route("/containers/{id}/raw/efficiency", get(K8sContainerMetricsController::get_metric_k8s_container_raw_efficiency))
        .route("/containers/cost", get(K8sContainerMetricsController::get_metric_k8s_containers_cost))
        .route("/containers/cost/summary", get(K8sContainerMetricsController::get_metric_k8s_containers_cost_summary))
        .route("/containers/cost/trend", get(K8sContainerMetricsController::get_metric_k8s_containers_cost_trend))
        .route("/containers/{id}/cost", get(K8sContainerMetricsController::get_metric_k8s_container_cost))
        .route("/containers/{id}/cost/summary", get(K8sContainerMetricsController::get_metric_k8s_container_cost_summary))
        .route("/containers/{id}/cost/trend", get(K8sContainerMetricsController::get_metric_k8s_container_cost_trend))

        // Namespaces
        .route("/namespaces/raw", get(K8sNamespaceMetricsController::get_metric_k8s_namespaces_raw))
        .route("/namespaces/raw/summary", get(K8sNamespaceMetricsController::get_metric_k8s_namespaces_raw_summary))
        .route("/namespaces/raw/efficiency", get(K8sNamespaceMetricsController::get_metric_k8s_namespaces_raw_efficiency))
        .route("/namespaces/{namespace}/raw", get(K8sNamespaceMetricsController::get_metric_k8s_namespace_raw))
        .route("/namespaces/{namespace}/raw/summary", get(K8sNamespaceMetricsController::get_metric_k8s_namespace_raw_summary))
        .route("/namespaces/{namespace}/raw/efficiency", get(K8sNamespaceMetricsController::get_metric_k8s_namespace_raw_efficiency))
        .route("/namespaces/cost", get(K8sNamespaceMetricsController::get_metric_k8s_namespaces_cost))
        .route("/namespaces/cost/summary", get(K8sNamespaceMetricsController::get_metric_k8s_namespaces_cost_summary))
        .route("/namespaces/cost/trend", get(K8sNamespaceMetricsController::get_metric_k8s_namespaces_cost_trend))
        .route("/namespaces/{namespace}/cost", get(K8sNamespaceMetricsController::get_metric_k8s_namespace_cost))
        .route("/namespaces/{namespace}/cost/summary", get(K8sNamespaceMetricsController::get_metric_k8s_namespace_cost_summary))
        .route("/namespaces/{namespace}/cost/trend", get(K8sNamespaceMetricsController::get_metric_k8s_namespace_cost_trend))

        // Deployments
        .route("/deployments/raw", get(K8sDeploymentMetricsController::get_metric_k8s_deployments_raw))
        .route("/deployments/raw/summary", get(K8sDeploymentMetricsController::get_metric_k8s_deployments_raw_summary))
        .route("/deployments/raw/efficiency", get(K8sDeploymentMetricsController::get_metric_k8s_deployments_raw_efficiency))
        .route("/deployments/{deployment}/raw", get(K8sDeploymentMetricsController::get_metric_k8s_deployment_raw))
        .route("/deployments/{deployment}/raw/summary", get(K8sDeploymentMetricsController::get_metric_k8s_deployment_raw_summary))
        .route("/deployments/{deployment}/raw/efficiency", get(K8sDeploymentMetricsController::get_metric_k8s_deployment_raw_efficiency))
        .route("/deployments/cost", get(K8sDeploymentMetricsController::get_metric_k8s_deployments_cost))
        .route("/deployments/cost/summary", get(K8sDeploymentMetricsController::get_metric_k8s_deployments_cost_summary))
        .route("/deployments/cost/trend", get(K8sDeploymentMetricsController::get_metric_k8s_deployments_cost_trend))
        .route("/deployments/{deployment}/cost", get(K8sDeploymentMetricsController::get_metric_k8s_deployment_cost))
        .route("/deployments/{deployment}/cost/summary", get(K8sDeploymentMetricsController::get_metric_k8s_deployment_cost_summary))
        .route("/deployments/{deployment}/cost/trend", get(K8sDeploymentMetricsController::get_metric_k8s_deployment_cost_trend))

        // Cluster
        .route("/cluster/raw", get(K8sClusterMetricsController::get_metric_k8s_cluster_raw))
        .route("/cluster/raw/summary", get(K8sClusterMetricsController::get_metric_k8s_cluster_raw_summary))
        .route("/cluster/raw/efficiency", get(K8sClusterMetricsController::get_metric_k8s_cluster_raw_efficiency))
        .route("/cluster/cost", get(K8sClusterMetricsController::get_metric_k8s_cluster_cost))
        .route("/cluster/cost/summary", get(K8sClusterMetricsController::get_metric_k8s_cluster_cost_summary))
        .route("/cluster/cost/trend", get(K8sClusterMetricsController::get_metric_k8s_cluster_cost_trend))
}
