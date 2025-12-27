//! Stored info routes (backed by persisted data)

use axum::{
    routing::{get, patch},
    Router,
};
use crate::api::controller::info::alerts::InfoAlertController;
use crate::api::controller::info::llm::InfoLlmController;
use crate::api::controller::info::info_controller::InfoController;
use crate::api::controller::info::k8s::{container, node, pod};
use crate::api::controller::info::setting::InfoSettingController;
use crate::app_state::AppState;

pub fn info_stored_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/settings",
            get(InfoSettingController::get_info_settings)
                .put(InfoSettingController::upsert_info_settings),
        )
        .route(
            "/alerts",
            get(InfoAlertController::get_info_alerts)
                .put(InfoAlertController::upsert_info_alerts),
        )
        .route(
            "/llm",
            get(InfoLlmController::get_info_llm)
                .put(InfoLlmController::upsert_info_llm),
        )
        .route(
            "/unit-prices",
            get(InfoController::get_info_unit_prices)
                .put(InfoController::upsert_info_unit_prices),
        )
        .route("/versions", get(InfoController::get_info_versions))
        .route(
            "/k8s/store/nodes",
            get(node::InfoK8sNodeController::list_k8s_nodes),
        )
        .route("/k8s/store/pods", get(pod::InfoK8sPodController::list_k8s_pods))
        .route(
            "/k8s/store/containers",
            get(container::InfoK8sContainerController::list_k8s_containers),
        )
        .route(
            "/k8s/store/nodes/{node_name}",
            get(node::InfoK8sNodeController::get_info_k8s_node),
        )
        .route(
            "/k8s/store/pods/{pod_uid}",
            get(pod::InfoK8sPodController::get_info_k8s_pod),
        )
        .route(
            "/k8s/store/containers/{id}",
            get(container::InfoK8sContainerController::get_info_k8s_container),
        )
        .route(
            "/k8s/store/nodes/{node_name}/filter",
            patch(node::InfoK8sNodeController::patch_info_k8s_node_filter),
        )
        .route(
            "/k8s/store/nodes/{node_name}/price",
            patch(node::InfoK8sNodeController::patch_info_k8s_node_price),
        )
        .route(
            "/k8s/store/pods/{pod_uid}",
            patch(pod::InfoK8sPodController::patch_info_k8s_pod),
        )
        .route(
            "/k8s/store/containers/{id}",
            patch(container::InfoK8sContainerController::patch_info_k8s_container),
        )
}
