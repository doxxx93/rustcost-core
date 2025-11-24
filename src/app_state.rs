use std::sync::Arc;

use crate::core::persistence::logs::log_repository::LogRepositoryImpl;
use crate::domain::system::service::log_service::LogService;

macro_rules! delegate_async_service {
    ($(fn $name:ident($($arg:ident : $typ:ty),*) -> $ret:ty => $path:path;)+) => {
        $(
            pub async fn $name(&self, $($arg: $typ),*) -> anyhow::Result<$ret> {
                $path($($arg),*).await
            }
        )+
    };
}

#[derive(Clone)]
pub struct AppState {
    pub log_service: Arc<LogService<LogRepositoryImpl>>,
    pub system_service: Arc<SystemService>,
    pub info_service: Arc<InfoService>,
    pub info_k8s_service: Arc<InfoK8sService>,
    pub metric_service: Arc<MetricService>,
}

pub fn build_app_state() -> AppState {
    AppState {
        log_service: Arc::new(LogService::new(LogRepositoryImpl::new())),
        system_service: Arc::new(SystemService::default()),
        info_service: Arc::new(InfoService::default()),
        info_k8s_service: Arc::new(InfoK8sService::default()),
        metric_service: Arc::new(MetricService::default()),
    }
}

#[derive(Clone, Default)]
pub struct SystemService;

impl SystemService {
    delegate_async_service! {
        fn status() -> serde_json::Value => crate::domain::system::service::status_service::status;
        fn health() -> serde_json::Value => crate::domain::system::service::health_service::health;
        fn backup() -> serde_json::Value => crate::domain::system::service::backup_service::backup;
        fn resync() -> serde_json::Value => crate::domain::system::service::resync_service::resync;
    }
}

#[derive(Clone, Default)]
pub struct InfoService;

impl InfoService {
    delegate_async_service! {
        fn get_info_unit_prices() -> crate::core::persistence::info::fixed::unit_price::info_unit_price_entity::InfoUnitPriceEntity => crate::domain::info::service::info_unit_price_service::get_info_unit_prices;
        fn upsert_info_unit_prices(req: crate::domain::info::dto::info_unit_price_upsert_request::InfoUnitPriceUpsertRequest) -> serde_json::Value => crate::domain::info::service::info_unit_price_service::upsert_info_unit_prices;
        fn get_info_versions() -> crate::core::persistence::info::fixed::version::info_version_entity::InfoVersionEntity => crate::domain::info::service::info_version_service::get_info_versions;
        fn get_info_settings() -> crate::core::persistence::info::fixed::setting::info_setting_entity::InfoSettingEntity => crate::domain::info::service::info_settings_service::get_info_settings;
        fn upsert_info_settings(req: crate::domain::info::dto::info_setting_upsert_request::InfoSettingUpsertRequest) -> serde_json::Value => crate::domain::info::service::info_settings_service::upsert_info_settings;
    }
}

#[derive(Clone, Default)]
pub struct InfoK8sService;

impl InfoK8sService {
    delegate_async_service! {
        fn get_k8s_namespaces() -> serde_json::Value => crate::domain::info::service::info_namespace_service::get_k8s_namespaces;
        fn get_k8s_deployments() -> serde_json::Value => crate::domain::info::service::info_k8s_deployment_service::get_k8s_deployments;
        fn get_k8s_persistent_volumes() -> serde_json::Value => crate::domain::info::service::info_k8s_persistent_volume_service::get_k8s_persistent_volumes;
        fn get_k8s_persistent_volume_claims() -> serde_json::Value => crate::domain::info::service::info_k8s_persistent_volume_claim_service::get_k8s_persistent_volume_claims;
        fn get_k8s_resource_quotas() -> serde_json::Value => crate::domain::info::service::info_k8s_resource_quota_service::get_k8s_resource_quotas;
        fn get_k8s_limit_ranges() -> serde_json::Value => crate::domain::info::service::info_k8s_limit_range_service::get_k8s_limit_ranges;
        fn get_k8s_hpas() -> serde_json::Value => crate::domain::info::service::info_k8s_hpa_service::get_k8s_hpas;
        fn get_info_k8s_node(node_name: String) -> crate::core::persistence::info::k8s::node::info_node_entity::InfoNodeEntity => crate::domain::info::service::info_k8s_node_service::get_info_k8s_node;
        fn list_k8s_nodes() -> Vec<crate::core::persistence::info::k8s::node::info_node_entity::InfoNodeEntity> => crate::domain::info::service::info_k8s_node_service::list_k8s_nodes;
        fn patch_info_k8s_node(id: String, patch: crate::domain::info::dto::info_k8s_node_patch_request::InfoK8sNodePatchRequest) -> serde_json::Value => crate::domain::info::service::info_k8s_node_service::patch_info_k8s_node;
        fn get_info_k8s_pod(pod_uid: String) -> crate::core::persistence::info::k8s::pod::info_pod_entity::InfoPodEntity => crate::domain::info::service::info_k8s_pod_service::get_info_k8s_pod;
        fn list_k8s_pods(filter: crate::api::dto::info_dto::K8sListQuery) -> Vec<crate::core::persistence::info::k8s::pod::info_pod_entity::InfoPodEntity> => crate::domain::info::service::info_k8s_pod_service::list_k8s_pods;
        fn patch_info_k8s_pod(id: String, payload: crate::domain::info::dto::info_k8s_pod_patch_request::InfoK8sPodPatchRequest) -> serde_json::Value => crate::domain::info::service::info_k8s_pod_service::patch_info_k8s_pod;
        fn get_info_k8s_container(id: String) -> crate::core::persistence::info::k8s::container::info_container_entity::InfoContainerEntity => crate::domain::info::service::info_k8s_container_service::get_info_k8s_container;
        fn list_k8s_containers(filter: crate::api::dto::info_dto::K8sListQuery) -> Vec<crate::core::persistence::info::k8s::container::info_container_entity::InfoContainerEntity> => crate::domain::info::service::info_k8s_container_service::list_k8s_containers;
        fn patch_info_k8s_container(id: String, payload: crate::domain::info::dto::info_k8s_container_patch_request::InfoK8sContainerPatchRequest) -> serde_json::Value => crate::domain::info::service::info_k8s_container_service::patch_info_k8s_container;
    }
}

#[derive(Clone, Default)]
pub struct MetricService;

impl MetricService {
    delegate_async_service! {
        fn get_metric_k8s_pods_raw(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::pod::service::get_metric_k8s_pods_raw;
        fn get_metric_k8s_pods_raw_summary(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::pod::service::get_metric_k8s_pods_raw_summary;
        fn get_metric_k8s_pods_raw_efficiency(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::pod::service::get_metric_k8s_pods_raw_efficiency;
        fn get_metric_k8s_pod_raw(pod_uid: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::pod::service::get_metric_k8s_pod_raw;
        fn get_metric_k8s_pod_raw_summary(pod_uid: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::pod::service::get_metric_k8s_pod_raw_summary;
        fn get_metric_k8s_pod_raw_efficiency(pod_uid: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::pod::service::get_metric_k8s_pod_raw_efficiency;
        fn get_metric_k8s_pods_cost(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::pod::service::get_metric_k8s_pods_cost;
        fn get_metric_k8s_pods_cost_summary(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::pod::service::get_metric_k8s_pods_cost_summary;
        fn get_metric_k8s_pods_cost_trend(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::pod::service::get_metric_k8s_pods_cost_trend;
        fn get_metric_k8s_pod_cost(pod_uid: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::pod::service::get_metric_k8s_pod_cost;
        fn get_metric_k8s_pod_cost_summary(pod_uid: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::pod::service::get_metric_k8s_pod_cost_summary;
        fn get_metric_k8s_pod_cost_trend(pod_uid: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::pod::service::get_metric_k8s_pod_cost_trend;
        fn get_metric_k8s_nodes_raw(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::node::service::get_metric_k8s_nodes_raw;
        fn get_metric_k8s_nodes_raw_summary(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::node::service::get_metric_k8s_nodes_raw_summary;
        fn get_metric_k8s_nodes_raw_efficiency(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::node::service::get_metric_k8s_nodes_raw_efficiency;
        fn get_metric_k8s_node_raw(node_name: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::node::service::get_metric_k8s_node_raw;
        fn get_metric_k8s_node_raw_summary(node_name: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::node::service::get_metric_k8s_node_raw_summary;
        fn get_metric_k8s_node_raw_efficiency(node_name: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::node::service::get_metric_k8s_node_raw_efficiency;
        fn get_metric_k8s_nodes_cost(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::node::service::get_metric_k8s_nodes_cost;
        fn get_metric_k8s_nodes_cost_summary(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::node::service::get_metric_k8s_nodes_cost_summary;
        fn get_metric_k8s_nodes_cost_trend(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::node::service::get_metric_k8s_nodes_cost_trend;
        fn get_metric_k8s_node_cost(node_name: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::node::service::get_metric_k8s_node_cost;
        fn get_metric_k8s_node_cost_summary(node_name: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::node::service::get_metric_k8s_node_cost_summary;
        fn get_metric_k8s_node_cost_trend(node_name: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::node::service::get_metric_k8s_node_cost_trend;
        fn get_metric_k8s_namespaces_raw(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::namespace::service::get_metric_k8s_namespaces_raw;
        fn get_metric_k8s_namespaces_raw_summary(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::namespace::service::get_metric_k8s_namespaces_raw_summary;
        fn get_metric_k8s_namespaces_raw_efficiency(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::namespace::service::get_metric_k8s_namespaces_raw_efficiency;
        fn get_metric_k8s_namespace_raw(namespace: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::namespace::service::get_metric_k8s_namespace_raw;
        fn get_metric_k8s_namespace_raw_summary(namespace: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::namespace::service::get_metric_k8s_namespace_raw_summary;
        fn get_metric_k8s_namespace_raw_efficiency(namespace: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::namespace::service::get_metric_k8s_namespace_raw_efficiency;
        fn get_metric_k8s_namespaces_cost(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::namespace::service::get_metric_k8s_namespaces_cost;
        fn get_metric_k8s_namespaces_cost_summary(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::namespace::service::get_metric_k8s_namespaces_cost_summary;
        fn get_metric_k8s_namespaces_cost_trend(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::namespace::service::get_metric_k8s_namespaces_cost_trend;
        fn get_metric_k8s_namespace_cost(namespace: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::namespace::service::get_metric_k8s_namespace_cost;
        fn get_metric_k8s_namespace_cost_summary(namespace: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::namespace::service::get_metric_k8s_namespace_cost_summary;
        fn get_metric_k8s_namespace_cost_trend(namespace: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::namespace::service::get_metric_k8s_namespace_cost_trend;
        fn get_metric_k8s_deployments_raw(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::deployment::service::get_metric_k8s_deployments_raw;
        fn get_metric_k8s_deployments_raw_summary(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::deployment::service::get_metric_k8s_deployments_raw_summary;
        fn get_metric_k8s_deployments_raw_efficiency(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::deployment::service::get_metric_k8s_deployments_raw_efficiency;
        fn get_metric_k8s_deployment_raw(deployment: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::deployment::service::get_metric_k8s_deployment_raw;
        fn get_metric_k8s_deployment_raw_summary(deployment: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::deployment::service::get_metric_k8s_deployment_raw_summary;
        fn get_metric_k8s_deployment_raw_efficiency(deployment: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::deployment::service::get_metric_k8s_deployment_raw_efficiency;
        fn get_metric_k8s_deployments_cost(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::deployment::service::get_metric_k8s_deployments_cost;
        fn get_metric_k8s_deployments_cost_summary(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::deployment::service::get_metric_k8s_deployments_cost_summary;
        fn get_metric_k8s_deployments_cost_trend(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::deployment::service::get_metric_k8s_deployments_cost_trend;
        fn get_metric_k8s_deployment_cost(deployment: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::deployment::service::get_metric_k8s_deployment_cost;
        fn get_metric_k8s_deployment_cost_summary(deployment: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::deployment::service::get_metric_k8s_deployment_cost_summary;
        fn get_metric_k8s_deployment_cost_trend(deployment: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::deployment::service::get_metric_k8s_deployment_cost_trend;
        fn get_metric_k8s_containers_raw(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::container::service::get_metric_k8s_containers_raw;
        fn get_metric_k8s_containers_raw_summary(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::container::service::get_metric_k8s_containers_raw_summary;
        fn get_metric_k8s_containers_raw_efficiency(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::container::service::get_metric_k8s_containers_raw_efficiency;
        fn get_metric_k8s_container_raw(id: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::container::service::get_metric_k8s_container_raw;
        fn get_metric_k8s_container_raw_summary(id: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::container::service::get_metric_k8s_container_raw_summary;
        fn get_metric_k8s_container_raw_efficiency(id: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::container::service::get_metric_k8s_container_raw_efficiency;
        fn get_metric_k8s_containers_cost(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::container::service::get_metric_k8s_containers_cost;
        fn get_metric_k8s_containers_cost_summary(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::container::service::get_metric_k8s_containers_cost_summary;
        fn get_metric_k8s_containers_cost_trend(q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::container::service::get_metric_k8s_containers_cost_trend;
        fn get_metric_k8s_container_cost(id: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::container::service::get_metric_k8s_container_cost;
        fn get_metric_k8s_container_cost_summary(id: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::container::service::get_metric_k8s_container_cost_summary;
        fn get_metric_k8s_container_cost_trend(id: String, q: crate::api::dto::metrics_dto::RangeQuery) -> serde_json::Value => crate::domain::metric::k8s::container::service::get_metric_k8s_container_cost_trend;
    }
}

impl MetricService {
    pub async fn get_metric_k8s_cluster_raw(
        &self,
        q: crate::api::dto::metrics_dto::RangeQuery,
    ) -> anyhow::Result<serde_json::Value> {
        let nodes = crate::domain::info::service::info_k8s_node_service::list_k8s_nodes().await?;
        crate::domain::metric::k8s::cluster::service::get_metric_k8s_cluster_raw(nodes, q).await
    }

    pub async fn get_metric_k8s_cluster_raw_summary(
        &self,
        q: crate::api::dto::metrics_dto::RangeQuery,
    ) -> anyhow::Result<serde_json::Value> {
        let nodes = crate::domain::info::service::info_k8s_node_service::list_k8s_nodes().await?;
        crate::domain::metric::k8s::cluster::service::get_metric_k8s_cluster_raw_summary(
            nodes, q,
        )
        .await
    }

    pub async fn get_metric_k8s_cluster_raw_efficiency(
        &self,
        q: crate::api::dto::metrics_dto::RangeQuery,
    ) -> anyhow::Result<serde_json::Value> {
        let nodes = crate::domain::info::service::info_k8s_node_service::list_k8s_nodes().await?;
        crate::domain::metric::k8s::cluster::service::get_metric_k8s_cluster_raw_efficiency(
            nodes, q,
        )
        .await
    }

    pub async fn get_metric_k8s_cluster_cost(
        &self,
        q: crate::api::dto::metrics_dto::RangeQuery,
    ) -> anyhow::Result<serde_json::Value> {
        let nodes = crate::domain::info::service::info_k8s_node_service::list_k8s_nodes().await?;
        let costs =
            crate::domain::info::service::info_unit_price_service::get_info_unit_prices().await?;
        crate::domain::metric::k8s::cluster::service::get_metric_k8s_cluster_cost(
            nodes, costs, q,
        )
        .await
    }

    pub async fn get_metric_k8s_cluster_cost_summary(
        &self,
        q: crate::api::dto::metrics_dto::RangeQuery,
    ) -> anyhow::Result<serde_json::Value> {
        let nodes = crate::domain::info::service::info_k8s_node_service::list_k8s_nodes().await?;
        let costs =
            crate::domain::info::service::info_unit_price_service::get_info_unit_prices().await?;
        crate::domain::metric::k8s::cluster::service::get_metric_k8s_cluster_cost_summary(
            nodes, costs, q,
        )
        .await
    }

    pub async fn get_metric_k8s_cluster_cost_trend(
        &self,
        q: crate::api::dto::metrics_dto::RangeQuery,
    ) -> anyhow::Result<serde_json::Value> {
        let nodes = crate::domain::info::service::info_k8s_node_service::list_k8s_nodes().await?;
        let costs =
            crate::domain::info::service::info_unit_price_service::get_info_unit_prices().await?;
        crate::domain::metric::k8s::cluster::service::get_metric_k8s_cluster_cost_trend(
            nodes, costs, q,
        )
        .await
    }
}
