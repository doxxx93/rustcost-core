//! Info routes (e.g., /api/v1/info/*)

use axum::{routing::get, Router};
use axum::routing::patch;
use crate::api::controller::info::info_controller::InfoController;
use crate::api::controller::info::setting::InfoSettingController;
use crate::api::controller::info::k8s::namespace::InfoK8sNamespaceController;
use crate::api::controller::info::k8s::deployment::InfoK8sDeploymentController;
use crate::api::controller::info::k8s::persistent_volume::InfoK8sPersistentVolumeController;
use crate::api::controller::info::k8s::pvc::InfoK8sPvcController;
use crate::api::controller::info::k8s::resource_quota::InfoK8sResourceQuotaController;
use crate::api::controller::info::k8s::limit_range::InfoK8sLimitRangeController;
use crate::api::controller::info::k8s::hpa::InfoK8sHpaController;
use crate::api::controller::info::k8s::{container, node, pod};
use crate::app_state::AppState;

pub fn info_routes() -> Router<AppState> {
    Router::new()
        .route("/settings", get(InfoSettingController::get_info_settings).put(InfoSettingController::upsert_info_settings))
        .route("/unit-prices", get(InfoController::get_info_unit_prices).put(InfoController::upsert_info_unit_prices))
        .route("/versions", get(InfoController::get_info_versions))

        .route("/k8s/namespaces", get(InfoK8sNamespaceController::get_k8s_namespaces))
        .route("/k8s/deployments", get(InfoK8sDeploymentController::get_k8s_deployments))
        .route("/k8s/persistentvolumes", get(InfoK8sPersistentVolumeController::get_k8s_persistent_volumes))
        .route("/k8s/persistentvolumeclaims", get(InfoK8sPvcController::get_k8s_persistent_volume_claims))
        .route("/k8s/resourcequotas", get(InfoK8sResourceQuotaController::get_k8s_resource_quotas))
        .route("/k8s/limitranges", get(InfoK8sLimitRangeController::get_k8s_limit_ranges))
        .route("/k8s/horizontalpodautoscalers", get(InfoK8sHpaController::get_k8s_hpas))
        .route("/k8s/nodes", get(node::InfoK8sNodeController::list_k8s_nodes))
        .route("/k8s/pods", get(pod::InfoK8sPodController::list_k8s_pods))
        .route("/k8s/containers", get(container::InfoK8sContainerController::list_k8s_containers))
        .route("/k8s/nodes/{node_name}", get(node::InfoK8sNodeController::get_info_k8s_node))
        .route("/k8s/pods/{pod_uid}", get(pod::InfoK8sPodController::get_info_k8s_pod))
        .route("/k8s/containers/{id}", get(container::InfoK8sContainerController::get_info_k8s_container))
        .route("/k8s/nodes/{node_name}", patch(node::InfoK8sNodeController::patch_info_k8s_node))
        .route("/k8s/pods/{pod_uid}", patch(pod::InfoK8sPodController::patch_info_k8s_pod))
        .route("/k8s/containers/{id}", patch(container::InfoK8sContainerController::patch_info_k8s_container))

}
