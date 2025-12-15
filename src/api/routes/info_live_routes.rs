//! Live info routes (proxied directly to Kubernetes)

use axum::{routing::get, Router};
use crate::api::controller::info::k8s::container::InfoK8sLiveContainerController;
use crate::api::controller::info::k8s::cronjob::InfoK8sCronJobController;
use crate::api::controller::info::k8s::daemonset::InfoK8sDaemonSetController;
use crate::api::controller::info::k8s::deployment::InfoK8sDeploymentController;
use crate::api::controller::info::k8s::hpa::InfoK8sHpaController;
use crate::api::controller::info::k8s::ingress::InfoK8sIngressController;
use crate::api::controller::info::k8s::job::InfoK8sJobController;
use crate::api::controller::info::k8s::limit_range::InfoK8sLimitRangeController;
use crate::api::controller::info::k8s::namespace::InfoK8sNamespaceController;
use crate::api::controller::info::k8s::node::InfoK8sLiveNodeController;
use crate::api::controller::info::k8s::pod::InfoK8sLivePodController;
use crate::api::controller::info::k8s::persistent_volume::InfoK8sPersistentVolumeController;
use crate::api::controller::info::k8s::pvc::InfoK8sPvcController;
use crate::api::controller::info::k8s::resource_quota::InfoK8sResourceQuotaController;
use crate::api::controller::info::k8s::service::InfoK8sServiceController;
use crate::api::controller::info::k8s::statefulset::InfoK8sStatefulSetController;
use crate::app_state::AppState;

pub fn info_live_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/k8s/live/namespaces",
            get(InfoK8sNamespaceController::get_k8s_namespaces),
        )
        .route(
            "/k8s/live/deployments/{namespace}/{name}",
            get(InfoK8sDeploymentController::get_k8s_deployment),
        )
        .route(
            "/k8s/live/deployments",
            get(InfoK8sDeploymentController::get_k8s_deployments),
        )
        .route(
            "/k8s/live/statefulsets/{namespace}/{name}",
            get(InfoK8sStatefulSetController::get_k8s_statefulset),
        )
        .route(
            "/k8s/live/statefulsets",
            get(InfoK8sStatefulSetController::get_k8s_statefulsets),
        )
        .route(
            "/k8s/live/daemonsets/{namespace}/{name}",
            get(InfoK8sDaemonSetController::get_k8s_daemonset),
        )
        .route(
            "/k8s/live/daemonsets",
            get(InfoK8sDaemonSetController::get_k8s_daemonsets),
        )
        .route(
            "/k8s/live/jobs/{namespace}/{name}",
            get(InfoK8sJobController::get_k8s_job),
        )
        .route("/k8s/live/jobs", get(InfoK8sJobController::get_k8s_jobs))
        .route(
            "/k8s/live/cronjobs/{namespace}/{name}",
            get(InfoK8sCronJobController::get_k8s_cronjob),
        )
        .route(
            "/k8s/live/cronjobs",
            get(InfoK8sCronJobController::get_k8s_cronjobs),
        )
        .route(
            "/k8s/live/services/{namespace}/{name}",
            get(InfoK8sServiceController::get_k8s_service),
        )
        .route(
            "/k8s/live/services",
            get(InfoK8sServiceController::get_k8s_services),
        )
        .route(
            "/k8s/live/ingresses/{namespace}/{name}",
            get(InfoK8sIngressController::get_k8s_ingress),
        )
        .route(
            "/k8s/live/ingresses",
            get(InfoK8sIngressController::get_k8s_ingresses),
        )
        .route(
            "/k8s/live/persistentvolumes",
            get(InfoK8sPersistentVolumeController::get_k8s_persistent_volumes),
        )
        .route(
            "/k8s/live/persistentvolumes/{name}",
            get(InfoK8sPersistentVolumeController::get_k8s_persistent_volume),
        )
        .route(
            "/k8s/live/persistentvolumeclaims",
            get(InfoK8sPvcController::get_k8s_persistent_volume_claims),
        )
        .route(
            "/k8s/live/persistentvolumeclaims/{namespace}/{name}",
            get(InfoK8sPvcController::get_k8s_persistent_volume_claim),
        )
        .route(
            "/k8s/live/resourcequotas",
            get(InfoK8sResourceQuotaController::get_k8s_resource_quotas),
        )
        .route(
            "/k8s/live/limitranges",
            get(InfoK8sLimitRangeController::get_k8s_limit_ranges),
        )
        .route(
            "/k8s/live/horizontalpodautoscalers",
            get(InfoK8sHpaController::get_k8s_hpas),
        )
        .route(
            "/k8s/live/nodes",
            get(InfoK8sLiveNodeController::list_k8s_nodes),
        )
        .route(
            "/k8s/live/nodes/{node_name}",
            get(InfoK8sLiveNodeController::get_k8s_node),
        )
        .route("/k8s/live/pods", get(InfoK8sLivePodController::list_k8s_pods))
        .route(
            "/k8s/live/pods/{pod_uid}",
            get(InfoK8sLivePodController::get_k8s_pod),
        )
        .route(
            "/k8s/live/containers",
            get(InfoK8sLiveContainerController::list_k8s_containers),
        )
        .route(
            "/k8s/live/containers/{id}",
            get(InfoK8sLiveContainerController::get_k8s_container),
        )
}
