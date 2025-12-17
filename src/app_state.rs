use std::sync::Arc;

//
// SHORT IMPORTS
//

// system
use crate::domain::system::service::status_service::status_internal;
use crate::domain::system::service::health_service::health;
use crate::domain::system::service::backup_service::backup;
use crate::domain::system::service::resync_service::resync;

// info
use crate::domain::info::service::info_unit_price_service::{
    get_info_unit_prices, upsert_info_unit_prices,
};
use crate::domain::info::service::info_version_service::get_info_versions;
use crate::domain::info::service::info_settings_service::{
    get_info_settings, upsert_info_settings,
};
use crate::domain::info::service::info_alerts_service::{
    get_info_alerts, upsert_info_alerts,
};

// info k8s
use crate::domain::info::service::info_namespace_service::get_k8s_namespaces;
use crate::domain::info::service::info_k8s_deployment_service::{
    get_k8s_deployment, get_k8s_deployments, get_k8s_deployments_paginated,
};
use crate::domain::info::service::info_k8s_statefulset_service::{
    get_k8s_statefulset, get_k8s_statefulsets, get_k8s_statefulsets_paginated,
};
use crate::domain::info::service::info_k8s_daemonset_service::{
    get_k8s_daemonset, get_k8s_daemonsets, get_k8s_daemonsets_paginated,
};
use crate::domain::info::service::info_k8s_job_service::{
    get_k8s_job, get_k8s_jobs, get_k8s_jobs_paginated,
};
use crate::domain::info::service::info_k8s_cronjob_service::{
    get_k8s_cronjob, get_k8s_cronjobs, get_k8s_cronjobs_paginated,
};
use crate::domain::info::service::info_k8s_service_service::{
    get_k8s_service, get_k8s_services, get_k8s_services_paginated,
};
use crate::domain::info::service::info_k8s_ingress_service::{
    get_k8s_ingress, get_k8s_ingresses, get_k8s_ingresses_paginated,
};
use crate::domain::info::service::info_k8s_persistent_volume_service::{
    get_k8s_persistent_volume, get_k8s_persistent_volumes, get_k8s_persistent_volumes_paginated,
};
use crate::domain::info::service::info_k8s_persistent_volume_claim_service::{
    get_k8s_persistent_volume_claim, get_k8s_persistent_volume_claims,
    get_k8s_persistent_volume_claims_paginated,
};
use crate::domain::info::service::info_k8s_resource_quota_service::get_k8s_resource_quotas;
use crate::domain::info::service::info_k8s_limit_range_service::get_k8s_limit_ranges;
use crate::domain::info::service::info_k8s_hpa_service::get_k8s_hpas;

use crate::domain::info::service::info_k8s_node_service::{
    get_info_k8s_node,
    list_k8s_nodes,
    patch_info_k8s_node_filter,
    patch_info_k8s_node_price,
};
use crate::domain::info::service::info_k8s_pod_service::{
    get_info_k8s_pod, list_k8s_pods, patch_info_k8s_pod,
};
use crate::domain::info::service::info_k8s_container_service::{
    get_info_k8s_container, list_k8s_containers, patch_info_k8s_container,
};
use crate::domain::info::service::info_k8s_live_node_service::{
    get_k8s_live_node,
    get_k8s_live_nodes_paginated,
};
use crate::domain::info::service::info_k8s_live_pod_service::{
    get_k8s_live_pod,
    get_k8s_live_pods_paginated,
};
use crate::domain::info::service::info_k8s_live_container_service::{
    get_k8s_live_container,
    get_k8s_live_containers_paginated,
};

// metrics
use crate::domain::metric::k8s::pod::service::*;
use crate::domain::metric::k8s::node::service::*;
use crate::domain::metric::k8s::namespace::service::*;
use crate::domain::metric::k8s::deployment::service::*;
use crate::domain::metric::k8s::container::service::*;
use crate::domain::metric::k8s::cluster::service::*;

// entities
use crate::core::persistence::info::fixed::unit_price::info_unit_price_entity::InfoUnitPriceEntity;
use crate::core::persistence::info::fixed::version::info_version_entity::InfoVersionEntity;
use crate::core::persistence::info::fixed::setting::info_setting_entity::InfoSettingEntity;
use crate::core::persistence::info::fixed::alerts::info_alert_entity::InfoAlertEntity;

use crate::core::persistence::info::k8s::node::info_node_entity::InfoNodeEntity;
use crate::core::persistence::info::k8s::pod::info_pod_entity::InfoPodEntity;
use crate::core::persistence::info::k8s::container::info_container_entity::InfoContainerEntity;

// dtos
use crate::domain::info::dto::info_unit_price_upsert_request::InfoUnitPriceUpsertRequest;
use crate::domain::info::dto::info_setting_upsert_request::InfoSettingUpsertRequest;
use crate::domain::info::dto::info_alert_upsert_request::InfoAlertUpsertRequest;
use crate::domain::info::dto::info_k8s_node_patch_request::{
    InfoK8sNodePatchRequest,
    InfoK8sNodePricePatchRequest,
};
use crate::domain::info::dto::info_k8s_pod_patch_request::InfoK8sPodPatchRequest;
use crate::domain::info::dto::info_k8s_container_patch_request::InfoK8sContainerPatchRequest;

use crate::api::dto::info_dto::{K8sListNodeQuery, K8sListQuery};
use crate::api::dto::k8s_pod_query_request_dto::K8sPodQueryRequestDto;
use crate::api::dto::paginated_response::PaginatedResponse;
use crate::api::dto::metrics_dto::RangeQuery;

// logs
use crate::core::persistence::logs::log_repository::LogRepositoryImpl;
use crate::core::state::runtime::alerts::alert_runtime_state_manager::AlertRuntimeStateManager;
use crate::core::state::runtime::alerts::alert_runtime_state_repository::AlertRuntimeStateRepository;
use crate::core::state::runtime::k8s::k8s_runtime_state_manager::K8sRuntimeStateManager;
use crate::core::state::runtime::k8s::k8s_runtime_state_repository::K8sRuntimeStateRepository;
use crate::domain::system::service::log_service::LogService;

//
// ============================================================
// CORE MACRO
// ============================================================
//
macro_rules! delegate_async_service {
    ($(fn $name:ident($($arg:ident : $typ:ty),*) -> $ret:ty => $path:path;)+) => {
        $(
            pub async fn $name(&self, $($arg: $typ),*) -> anyhow::Result<$ret> {
                $path($($arg),*).await
            }
        )+
    };
    ($(fn $name:ident($($arg:ident : $typ:ty),*) -> $ret:ty => $expr:expr;)+) => {
        $(
            pub async fn $name(&self, $($arg: $typ),*) -> anyhow::Result<$ret> {
                $expr.await
            }
        )+
    };
}

//
// ============================================================
// APP STATE
// ============================================================
//
#[derive(Clone)]
pub struct AppState {
    pub log_service: Arc<LogService<LogRepositoryImpl>>,
    pub system_service: Arc<SystemService>,
    pub info_service: Arc<InfoService>,
    pub info_k8s_service: Arc<InfoK8sService>,
    pub metric_service: Arc<MetricService>,

    // runtime state managers
    pub k8s_state: Arc<K8sRuntimeStateManager<K8sRuntimeStateRepository>>,
    pub alerts: Arc<AlertRuntimeStateManager<AlertRuntimeStateRepository>>
}

pub fn build_app_state() -> AppState {
    // Create repositories
    let k8s_repo = K8sRuntimeStateRepository::new().shared();
    let alert_repo = AlertRuntimeStateRepository::new().shared();

    // Managers wrap repositories
    let k8s_state = Arc::new(K8sRuntimeStateManager::new(k8s_repo));
    let alerts = Arc::new(AlertRuntimeStateManager::new(alert_repo));

    AppState {
        log_service: Arc::new(LogService::new(LogRepositoryImpl::new())),
        system_service: Arc::new(SystemService::new(k8s_state.clone())),
        info_service: Arc::new(InfoService::default()),
        info_k8s_service: Arc::new(InfoK8sService::default()),
        metric_service: Arc::new(MetricService::default()),

        k8s_state,
        alerts,
    }
}

//
// ============================================================
// SYSTEM
// ============================================================
//
#[derive(Clone)]
pub struct SystemService {
    pub k8s_state: Arc<K8sRuntimeStateManager<K8sRuntimeStateRepository>>,
}

impl SystemService {
    pub fn new(k8s_state: Arc<K8sRuntimeStateManager<K8sRuntimeStateRepository>>) -> Self {
        Self { k8s_state }
    }

    delegate_async_service! {
        fn health() -> serde_json::Value => health;
        fn backup() -> serde_json::Value => backup;
    }
    pub async fn status(&self) -> anyhow::Result<serde_json::Value> {
        status_internal(self.k8s_state.clone()).await
    }
    pub async fn resync(&self) -> anyhow::Result<serde_json::Value> {
        resync(self.k8s_state.clone()).await
    }
}

//
// ============================================================
// INFO
// ============================================================
//
#[derive(Clone, Default)]
pub struct InfoService;

impl InfoService {
    delegate_async_service! {
        fn get_info_unit_prices() -> InfoUnitPriceEntity => get_info_unit_prices;
        fn upsert_info_unit_prices(req: InfoUnitPriceUpsertRequest) -> serde_json::Value => upsert_info_unit_prices;

        fn get_info_versions() -> InfoVersionEntity => get_info_versions;

        fn get_info_alerts() -> InfoAlertEntity => get_info_alerts;
        fn upsert_info_alerts(req: InfoAlertUpsertRequest) -> serde_json::Value => upsert_info_alerts;

        fn get_info_settings() -> InfoSettingEntity => get_info_settings;
        fn upsert_info_settings(req: InfoSettingUpsertRequest) -> serde_json::Value => upsert_info_settings;
    }
}

//
// ============================================================
// INFO K8S
// ============================================================
//
#[derive(Clone, Default)]
pub struct InfoK8sService;

impl InfoK8sService {
    delegate_async_service! {
        fn get_k8s_namespaces() -> serde_json::Value => get_k8s_namespaces;
        fn get_k8s_deployments() -> crate::api::dto::paginated_response::PaginatedResponse<k8s_openapi::api::apps::v1::Deployment> => get_k8s_deployments;
        fn get_k8s_deployments_paginated(limit: Option<usize>, offset: Option<usize>) -> PaginatedResponse<k8s_openapi::api::apps::v1::Deployment> => get_k8s_deployments_paginated;
        fn get_k8s_deployment(namespace: String, name: String) -> k8s_openapi::api::apps::v1::Deployment => get_k8s_deployment;
        fn get_k8s_statefulsets() -> crate::api::dto::paginated_response::PaginatedResponse<k8s_openapi::api::apps::v1::StatefulSet> => get_k8s_statefulsets;
        fn get_k8s_statefulsets_paginated(limit: Option<usize>, offset: Option<usize>) -> PaginatedResponse<k8s_openapi::api::apps::v1::StatefulSet> => get_k8s_statefulsets_paginated;
        fn get_k8s_statefulset(namespace: String, name: String) -> k8s_openapi::api::apps::v1::StatefulSet => get_k8s_statefulset;
        fn get_k8s_daemonsets() -> crate::api::dto::paginated_response::PaginatedResponse<k8s_openapi::api::apps::v1::DaemonSet> => get_k8s_daemonsets;
        fn get_k8s_daemonsets_paginated(limit: Option<usize>, offset: Option<usize>) -> PaginatedResponse<k8s_openapi::api::apps::v1::DaemonSet> => get_k8s_daemonsets_paginated;
        fn get_k8s_daemonset(namespace: String, name: String) -> k8s_openapi::api::apps::v1::DaemonSet => get_k8s_daemonset;

        fn get_k8s_jobs() -> crate::api::dto::paginated_response::PaginatedResponse<k8s_openapi::api::batch::v1::Job> => get_k8s_jobs;
        fn get_k8s_jobs_paginated(limit: Option<usize>, offset: Option<usize>) -> PaginatedResponse<k8s_openapi::api::batch::v1::Job> => get_k8s_jobs_paginated;
        fn get_k8s_job(namespace: String, name: String) -> k8s_openapi::api::batch::v1::Job => get_k8s_job;

        fn get_k8s_cronjobs() -> crate::api::dto::paginated_response::PaginatedResponse<k8s_openapi::api::batch::v1::CronJob> => get_k8s_cronjobs;
        fn get_k8s_cronjobs_paginated(limit: Option<usize>, offset: Option<usize>) -> PaginatedResponse<k8s_openapi::api::batch::v1::CronJob> => get_k8s_cronjobs_paginated;
        fn get_k8s_cronjob(namespace: String, name: String) -> k8s_openapi::api::batch::v1::CronJob => get_k8s_cronjob;

        fn get_k8s_services() -> crate::api::dto::paginated_response::PaginatedResponse<k8s_openapi::api::core::v1::Service> => get_k8s_services;
        fn get_k8s_services_paginated(limit: Option<usize>, offset: Option<usize>) -> PaginatedResponse<k8s_openapi::api::core::v1::Service> => get_k8s_services_paginated;
        fn get_k8s_service(namespace: String, name: String) -> k8s_openapi::api::core::v1::Service => get_k8s_service;

        fn get_k8s_ingresses() -> crate::api::dto::paginated_response::PaginatedResponse<k8s_openapi::api::networking::v1::Ingress> => get_k8s_ingresses;
        fn get_k8s_ingresses_paginated(limit: Option<usize>, offset: Option<usize>) -> PaginatedResponse<k8s_openapi::api::networking::v1::Ingress> => get_k8s_ingresses_paginated;
        fn get_k8s_ingress(namespace: String, name: String) -> k8s_openapi::api::networking::v1::Ingress => get_k8s_ingress;

        fn get_k8s_persistent_volumes() -> crate::api::dto::paginated_response::PaginatedResponse<k8s_openapi::api::core::v1::PersistentVolume> => get_k8s_persistent_volumes;
        fn get_k8s_persistent_volumes_paginated(limit: Option<usize>, offset: Option<usize>) -> PaginatedResponse<k8s_openapi::api::core::v1::PersistentVolume> => get_k8s_persistent_volumes_paginated;
        fn get_k8s_persistent_volume(name: String) -> k8s_openapi::api::core::v1::PersistentVolume => get_k8s_persistent_volume;

        fn get_k8s_persistent_volume_claims() -> crate::api::dto::paginated_response::PaginatedResponse<k8s_openapi::api::core::v1::PersistentVolumeClaim> => get_k8s_persistent_volume_claims;
        fn get_k8s_persistent_volume_claims_paginated(limit: Option<usize>, offset: Option<usize>) -> PaginatedResponse<k8s_openapi::api::core::v1::PersistentVolumeClaim> => get_k8s_persistent_volume_claims_paginated;
        fn get_k8s_persistent_volume_claim(namespace: String, name: String) -> k8s_openapi::api::core::v1::PersistentVolumeClaim => get_k8s_persistent_volume_claim;
        fn get_k8s_resource_quotas() -> serde_json::Value => get_k8s_resource_quotas;
        fn get_k8s_limit_ranges() -> serde_json::Value => get_k8s_limit_ranges;
        fn get_k8s_hpas() -> serde_json::Value => get_k8s_hpas;

        fn get_k8s_live_nodes_paginated(limit: Option<usize>, offset: Option<usize>) -> PaginatedResponse<k8s_openapi::api::core::v1::Node> => get_k8s_live_nodes_paginated;
        fn get_k8s_live_node(node_name: String) -> k8s_openapi::api::core::v1::Node => get_k8s_live_node;

        fn get_k8s_live_pods_paginated(limit: Option<usize>, offset: Option<usize>) -> PaginatedResponse<k8s_openapi::api::core::v1::Pod> => get_k8s_live_pods_paginated;
        fn get_k8s_live_pod(pod_uid: String) -> k8s_openapi::api::core::v1::Pod => get_k8s_live_pod;

        fn get_k8s_live_containers_paginated(limit: Option<usize>, offset: Option<usize>) -> PaginatedResponse<InfoContainerEntity> => get_k8s_live_containers_paginated;
        fn get_k8s_live_container(id: String) -> InfoContainerEntity => get_k8s_live_container;

        fn get_info_k8s_node(node_name: String) -> InfoNodeEntity => get_info_k8s_node;
        fn list_k8s_nodes(filter: K8sListNodeQuery) -> Vec<InfoNodeEntity> => list_k8s_nodes;
        fn patch_info_k8s_node_filter(id: String, patch: InfoK8sNodePatchRequest) -> serde_json::Value => patch_info_k8s_node_filter;
        fn patch_info_k8s_node_price(id: String, patch: InfoK8sNodePricePatchRequest) -> serde_json::Value => patch_info_k8s_node_price;

        fn get_info_k8s_pod(pod_uid: String) -> InfoPodEntity => get_info_k8s_pod;
        fn list_k8s_pods(state: AppState, filter: K8sPodQueryRequestDto) -> PaginatedResponse<InfoPodEntity> => list_k8s_pods;
        fn patch_info_k8s_pod(id: String, payload: InfoK8sPodPatchRequest) -> serde_json::Value => patch_info_k8s_pod;

        fn get_info_k8s_container(id: String) -> InfoContainerEntity => get_info_k8s_container;
        fn list_k8s_containers(filter: K8sListQuery) -> Vec<InfoContainerEntity> => list_k8s_containers;
        fn patch_info_k8s_container(id: String, payload: InfoK8sContainerPatchRequest) -> serde_json::Value => patch_info_k8s_container;
    }
}

//
// ============================================================
// METRICS
// ============================================================
//
#[derive(Clone, Default)]
pub struct MetricService;

impl MetricService {
    delegate_async_service! {
        fn get_metric_k8s_pods_raw(q: RangeQuery, pod_uids: Vec<String>) -> serde_json::Value => get_metric_k8s_pods_raw;
        fn get_metric_k8s_pods_raw_summary(q: RangeQuery, pod_uids: Vec<String>) -> serde_json::Value => get_metric_k8s_pods_raw_summary;
        fn get_metric_k8s_pods_raw_efficiency(q: RangeQuery, _pod_uids: Vec<String>) -> serde_json::Value => get_metric_k8s_pods_raw_efficiency;

        fn get_metric_k8s_pod_raw(pod_uid: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_pod_raw;
        fn get_metric_k8s_pod_raw_summary(pod_uid: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_pod_raw_summary;
        fn get_metric_k8s_pod_raw_efficiency(pod_uid: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_pod_raw_efficiency;

        fn get_metric_k8s_pods_cost(q: RangeQuery, _pod_uids: Vec<String>) -> serde_json::Value => get_metric_k8s_pods_cost;
        fn get_metric_k8s_pods_cost_summary(q: RangeQuery, _pod_uids: Vec<String>) -> serde_json::Value => get_metric_k8s_pods_cost_summary;
        fn get_metric_k8s_pods_cost_trend(q: RangeQuery, _pod_uids: Vec<String>) -> serde_json::Value => get_metric_k8s_pods_cost_trend;

        fn get_metric_k8s_pod_cost(pod_uid: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_pod_cost;
        fn get_metric_k8s_pod_cost_summary(pod_uid: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_pod_cost_summary;
        fn get_metric_k8s_pod_cost_trend(pod_uid: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_pod_cost_trend;

        fn get_metric_k8s_nodes_raw(q: RangeQuery, node_names: Vec<String>) -> serde_json::Value => get_metric_k8s_nodes_raw;
        fn get_metric_k8s_nodes_raw_summary(q: RangeQuery, node_names: Vec<String>) -> serde_json::Value => get_metric_k8s_nodes_raw_summary;
        fn get_metric_k8s_nodes_raw_efficiency(q: RangeQuery, node_names: Vec<String>) -> serde_json::Value => get_metric_k8s_nodes_raw_efficiency;

        fn get_metric_k8s_node_raw(node_name: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_node_raw;
        fn get_metric_k8s_node_raw_summary(node_name: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_node_raw_summary;
        fn get_metric_k8s_node_raw_efficiency(node_name: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_node_raw_efficiency;

        fn get_metric_k8s_nodes_cost(q: RangeQuery, node_names: Vec<String>) -> serde_json::Value => get_metric_k8s_nodes_cost;
        fn get_metric_k8s_nodes_cost_summary(q: RangeQuery, node_names: Vec<String>) -> serde_json::Value => get_metric_k8s_nodes_cost_summary;
        fn get_metric_k8s_nodes_cost_trend(q: RangeQuery, node_names: Vec<String>) -> serde_json::Value => get_metric_k8s_nodes_cost_trend;

        fn get_metric_k8s_node_cost(node_name: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_node_cost;
        fn get_metric_k8s_node_cost_summary(node_name: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_node_cost_summary;
        fn get_metric_k8s_node_cost_trend(node_name: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_node_cost_trend;

        fn get_metric_k8s_namespaces_raw(q: RangeQuery, namespaces: Vec<String>) -> serde_json::Value => get_metric_k8s_namespaces_raw;
        fn get_metric_k8s_namespaces_raw_summary(q: RangeQuery, namespaces: Vec<String>) -> serde_json::Value => get_metric_k8s_namespaces_raw_summary;
        fn get_metric_k8s_namespaces_raw_efficiency(q: RangeQuery, namespaces: Vec<String>) -> serde_json::Value => get_metric_k8s_namespaces_raw_efficiency;

        fn get_metric_k8s_namespace_raw(ns: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_namespace_raw;
        fn get_metric_k8s_namespace_raw_summary(ns: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_namespace_raw_summary;
        fn get_metric_k8s_namespace_raw_efficiency(ns: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_namespace_raw_efficiency;

        fn get_metric_k8s_namespaces_cost(q: RangeQuery, namespaces: Vec<String>) -> serde_json::Value => get_metric_k8s_namespaces_cost;
        fn get_metric_k8s_namespaces_cost_summary(q: RangeQuery, namespaces: Vec<String>) -> serde_json::Value => get_metric_k8s_namespaces_cost_summary;
        fn get_metric_k8s_namespaces_cost_trend(q: RangeQuery, namespaces: Vec<String>) -> serde_json::Value => get_metric_k8s_namespaces_cost_trend;

        fn get_metric_k8s_namespace_cost(ns: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_namespace_cost;
        fn get_metric_k8s_namespace_cost_summary(ns: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_namespace_cost_summary;
        fn get_metric_k8s_namespace_cost_trend(ns: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_namespace_cost_trend;

        fn get_metric_k8s_deployments_raw(q: RangeQuery, deployments: Vec<String>) -> serde_json::Value => get_metric_k8s_deployments_raw;
        fn get_metric_k8s_deployments_raw_summary(q: RangeQuery, deployments: Vec<String>) -> serde_json::Value => get_metric_k8s_deployments_raw_summary;
        fn get_metric_k8s_deployments_raw_efficiency(q: RangeQuery, deployments: Vec<String>) -> serde_json::Value => get_metric_k8s_deployments_raw_efficiency;

        fn get_metric_k8s_deployment_raw(name: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_deployment_raw;
        fn get_metric_k8s_deployment_raw_summary(name: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_deployment_raw_summary;
        fn get_metric_k8s_deployment_raw_efficiency(name: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_deployment_raw_efficiency;

        fn get_metric_k8s_deployments_cost(q: RangeQuery, deployments: Vec<String>) -> serde_json::Value => get_metric_k8s_deployments_cost;
        fn get_metric_k8s_deployments_cost_summary(q: RangeQuery, deployments: Vec<String>) -> serde_json::Value => get_metric_k8s_deployments_cost_summary;
        fn get_metric_k8s_deployments_cost_trend(q: RangeQuery, deployments: Vec<String>) -> serde_json::Value => get_metric_k8s_deployments_cost_trend;

        fn get_metric_k8s_deployment_cost(name: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_deployment_cost;
        fn get_metric_k8s_deployment_cost_summary(name: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_deployment_cost_summary;
        fn get_metric_k8s_deployment_cost_trend(name: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_deployment_cost_trend;

        fn get_metric_k8s_containers_raw(q: RangeQuery, container_keys: Vec<String>) -> serde_json::Value => get_metric_k8s_containers_raw;
        fn get_metric_k8s_containers_raw_summary(q: RangeQuery, container_keys: Vec<String>) -> serde_json::Value => get_metric_k8s_containers_raw_summary;
        fn get_metric_k8s_containers_raw_efficiency(q: RangeQuery, container_keys: Vec<String>) -> serde_json::Value => get_metric_k8s_containers_raw_efficiency;

        fn get_metric_k8s_container_raw(id: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_container_raw;
        fn get_metric_k8s_container_raw_summary(id: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_container_raw_summary;
        fn get_metric_k8s_container_raw_efficiency(id: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_container_raw_efficiency;

        fn get_metric_k8s_containers_cost(q: RangeQuery, container_keys: Vec<String>) -> serde_json::Value => get_metric_k8s_containers_cost;
        fn get_metric_k8s_containers_cost_summary(q: RangeQuery, container_keys: Vec<String>) -> serde_json::Value => get_metric_k8s_containers_cost_summary;
        fn get_metric_k8s_containers_cost_trend(q: RangeQuery, container_keys: Vec<String>) -> serde_json::Value => get_metric_k8s_containers_cost_trend;

        fn get_metric_k8s_container_cost(id: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_container_cost;
        fn get_metric_k8s_container_cost_summary(id: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_container_cost_summary;
        fn get_metric_k8s_container_cost_trend(id: String, q: RangeQuery) -> serde_json::Value => get_metric_k8s_container_cost_trend;
    }
}

//
// ============================================================
// METRIC CLUSTER (manual)
// ============================================================
//
impl MetricService {
    pub async fn get_metric_k8s_cluster_raw(
        &self,
        q: RangeQuery,
        node_names: Vec<String>
    ) -> anyhow::Result<serde_json::Value> {
        get_metric_k8s_cluster_raw(node_names, q).await
    }

    pub async fn get_metric_k8s_cluster_raw_summary(
        &self,
        q: RangeQuery,
        node_names: Vec<String>
    ) -> anyhow::Result<serde_json::Value> {
        get_metric_k8s_cluster_raw_summary(node_names, q).await
    }

    pub async fn get_metric_k8s_cluster_raw_efficiency(
        &self,
        q: RangeQuery,
        node_names: Vec<String>
    ) -> anyhow::Result<serde_json::Value> {
        let nodes = list_k8s_nodes(K8sListNodeQuery::default()).await?;
        get_metric_k8s_cluster_raw_efficiency(nodes, node_names, q).await
    }

    pub async fn get_metric_k8s_cluster_cost(
        &self,
        q: RangeQuery,
        node_names: Vec<String>,
    ) -> anyhow::Result<serde_json::Value> {
        let costs = get_info_unit_prices().await?;
        get_metric_k8s_cluster_cost(node_names, costs, q).await
    }

    pub async fn get_metric_k8s_cluster_cost_summary(
        &self,
        q: RangeQuery,
        node_names: Vec<String>,
    ) -> anyhow::Result<serde_json::Value> {
        let costs = get_info_unit_prices().await?;
        get_metric_k8s_cluster_cost_summary(node_names, costs, q).await
    }

    pub async fn get_metric_k8s_cluster_cost_trend(
        &self,
        q: RangeQuery,
        node_names: Vec<String>,
    ) -> anyhow::Result<serde_json::Value> {
        let costs = get_info_unit_prices().await?;
        get_metric_k8s_cluster_cost_trend(node_names, costs, q).await
    }
}
