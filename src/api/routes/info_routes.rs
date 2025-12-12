//! Info routes (e.g., /api/v1/info/*)

use axum::{routing::get, Router};
use axum::routing::patch;
use crate::api::controller::info::info_controller::InfoController;
use crate::api::controller::info::setting::InfoSettingController;
use crate::api::controller::info::k8s::namespace::InfoK8sNamespaceController;
use crate::api::controller::info::k8s::deployment::InfoK8sDeploymentController;
use crate::api::controller::info::k8s::statefulset::InfoK8sStatefulSetController;
use crate::api::controller::info::k8s::daemonset::InfoK8sDaemonSetController;
use crate::api::controller::info::k8s::job::InfoK8sJobController;
use crate::api::controller::info::k8s::cronjob::InfoK8sCronJobController;
use crate::api::controller::info::k8s::service::InfoK8sServiceController;
use crate::api::controller::info::k8s::ingress::InfoK8sIngressController;
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
        .route(
            "/k8s/deployments/{namespace}/{name}",
            get(InfoK8sDeploymentController::get_k8s_deployment),
        )
        .route("/k8s/deployments", get(InfoK8sDeploymentController::get_k8s_deployments))
        .route(
            "/k8s/statefulsets/{namespace}/{name}",
            get(InfoK8sStatefulSetController::get_k8s_statefulset),
        )
        .route("/k8s/statefulsets", get(InfoK8sStatefulSetController::get_k8s_statefulsets))
        .route(
            "/k8s/daemonsets/{namespace}/{name}",
            get(InfoK8sDaemonSetController::get_k8s_daemonset),
        )
        .route("/k8s/daemonsets", get(InfoK8sDaemonSetController::get_k8s_daemonsets))
        .route("/k8s/jobs/{namespace}/{name}", get(InfoK8sJobController::get_k8s_job))
        .route("/k8s/jobs", get(InfoK8sJobController::get_k8s_jobs))
        .route(
            "/k8s/cronjobs/{namespace}/{name}",
            get(InfoK8sCronJobController::get_k8s_cronjob),
        )
        .route("/k8s/cronjobs", get(InfoK8sCronJobController::get_k8s_cronjobs))
        .route(
            "/k8s/services/{namespace}/{name}",
            get(InfoK8sServiceController::get_k8s_service),
        )
        .route("/k8s/services", get(InfoK8sServiceController::get_k8s_services))
        .route(
            "/k8s/ingresses/{namespace}/{name}",
            get(InfoK8sIngressController::get_k8s_ingress),
        )
        .route("/k8s/ingresses", get(InfoK8sIngressController::get_k8s_ingresses))
        .route("/k8s/persistentvolumes", get(InfoK8sPersistentVolumeController::get_k8s_persistent_volumes))
        .route(
            "/k8s/persistentvolumes/{name}",
            get(InfoK8sPersistentVolumeController::get_k8s_persistent_volume),
        )
        .route("/k8s/persistentvolumeclaims", get(InfoK8sPvcController::get_k8s_persistent_volume_claims))
        .route(
            "/k8s/persistentvolumeclaims/{namespace}/{name}",
            get(InfoK8sPvcController::get_k8s_persistent_volume_claim),
        )
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
