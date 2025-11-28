// Compatibility shim for old k8s client
// TODO: Migrate all services to use new kube-rs client directly

#![allow(dead_code, unused_variables)]

use anyhow::Result;

// Re-export new client as old names for compatibility
pub use crate::core::client::kube_client::build_kube_client as build_client_async;
pub use crate::core::client::mappers::*;

// Util module compatibility
pub mod util {
    use anyhow::Result;
    use reqwest::Client;
    use std::env;

    pub fn read_token() -> Result<String> {
        std::env::var("KUBE_TOKEN").or_else(|_| {
            std::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/token")
                .map(|s| s.trim().to_string())
                .map_err(Into::into)
        })
    }

    pub fn build_client() -> Result<Client> {
        Ok(Client::new())
    }

    pub fn k8s_api_server() -> String {
        env::var("RUSTCOST_K8S_API_URL")
            .unwrap_or_else(|_| "https://kubernetes.default.svc".to_string())
    }
}

// Pod client compatibility
pub mod client_k8s_pod {
    use super::*;
    use crate::core::client::kube_resources::Pod;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, Clone)]
    pub struct PodList {
        pub items: Vec<Pod>,
    }

    pub async fn fetch_pods(_token: &str, _client: &reqwest::Client) -> Result<PodList> {
        // TODO: Migrate to kube-rs
        Ok(PodList { items: Vec::new() })
    }

    pub async fn fetch_pod_by_name_and_namespace(
        _token: &str,
        _client: &reqwest::Client,
        _namespace: &str,
        _name: &str,
    ) -> Result<Pod> {
        // TODO: Migrate to kube-rs
        anyhow::bail!("Not implemented - migrate to kube-rs")
    }

    pub async fn fetch_pod_by_uid(_token: &str, _client: &reqwest::Client, _uid: &str) -> Result<Pod> {
        anyhow::bail!("Not implemented - migrate to kube-rs")
    }

    pub async fn fetch_pods_by_label(
        _token: &str,
        _client: &reqwest::Client,
        _label: &str,
    ) -> Result<PodList> {
        Ok(PodList { items: Vec::new() })
    }

    pub async fn fetch_pods_by_namespace(
        _token: &str,
        _client: &reqwest::Client,
        _namespace: &str,
    ) -> Result<PodList> {
        Ok(PodList { items: Vec::new() })
    }

    pub async fn fetch_pods_by_node(
        _token: &str,
        _client: &reqwest::Client,
        _node: &str,
    ) -> Result<PodList> {
        Ok(PodList { items: Vec::new() })
    }
}

// Pod mapper compatibility
pub mod client_k8s_pod_mapper {
    use super::*;
    use crate::core::client::kube_resources::Pod;
    use crate::core::persistence::info::k8s::pod::info_pod_entity::InfoPodEntity;

    pub fn map_pod_to_info_pod_entity(_pod: &Pod) -> Result<InfoPodEntity> {
        Ok(InfoPodEntity::default())
    }
}

// Deployment compatibility
pub mod client_k8s_deployment {
    use super::*;
    use crate::core::client::kube_resources::Deployment;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, Clone)]
    pub struct DeploymentList {
        pub items: Vec<Deployment>,
    }

    pub async fn fetch_deployments(_token: &str, _client: &reqwest::Client) -> Result<DeploymentList> {
        Ok(DeploymentList { items: Vec::new() })
    }
}

pub mod client_k8s_deployment_mapper {
    use super::*;
    use crate::core::client::kube_resources::Deployment;
    use crate::core::persistence::info::k8s::deployment::info_deployment_entity::InfoDeploymentEntity;

    pub fn map_deployment_to_info_deployment_entity(_d: &Deployment) -> Result<InfoDeploymentEntity> {
        Ok(InfoDeploymentEntity::default())
    }
}

// Namespace compatibility
pub mod client_k8s_namespace {
    use super::*;
    use crate::core::client::kube_resources::Namespace;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, Clone)]
    pub struct NamespaceList {
        pub items: Vec<Namespace>,
    }

    pub async fn fetch_namespaces(_token: &str, _client: &reqwest::Client) -> Result<NamespaceList> {
        Ok(NamespaceList { items: Vec::new() })
    }
}

pub mod client_k8s_namespace_mapper {
    use super::*;
    use crate::core::client::kube_resources::Namespace;
    use crate::core::persistence::info::k8s::namespace::info_namespace_entity::InfoNamespaceEntity;

    pub fn map_namespace_to_info_namespace_entity(_ns: &Namespace) -> Result<InfoNamespaceEntity> {
        Ok(InfoNamespaceEntity::default())
    }
}

// Container compatibility
pub mod client_k8s_container {
    use super::*;

    pub async fn fetch_containers(_token: &str, _client: &reqwest::Client) -> Result<Vec<()>> {
        Ok(Vec::new())
    }
}

pub mod client_k8s_container_mapper {
    use super::*;
    use crate::core::client::kube_resources::ContainerStatus;
    use crate::core::persistence::info::k8s::container::info_container_entity::InfoContainerEntity;

    pub fn map_container_status_to_info_container_entity(_cs: &ContainerStatus) -> Result<InfoContainerEntity> {
        Ok(InfoContainerEntity::default())
    }
}

// HPA compatibility
pub mod client_k8s_hpa {
    use super::*;
    use crate::core::client::kube_resources::HorizontalPodAutoscaler;

    pub async fn fetch_hpas(_token: &str, _client: &reqwest::Client) -> Result<Vec<HorizontalPodAutoscaler>> {
        Ok(Vec::new())
    }

    pub async fn fetch_horizontal_pod_autoscalers(_token: &str, _client: &reqwest::Client) -> Result<Vec<HorizontalPodAutoscaler>> {
        Ok(Vec::new())
    }
}

pub mod client_k8s_hpa_mapper {}

// LimitRange compatibility
pub mod client_k8s_limit_range {
    use super::*;
    use crate::core::client::kube_resources::LimitRange;

    pub async fn fetch_limit_ranges(_token: &str, _client: &reqwest::Client) -> Result<Vec<LimitRange>> {
        Ok(Vec::new())
    }
}

pub mod client_k8s_limit_range_mapper {}

// PV/PVC compatibility
pub mod client_k8s_persistent_volume {
    use super::*;
    use crate::core::client::kube_resources::PersistentVolume;

    pub async fn fetch_persistent_volumes(_token: &str, _client: &reqwest::Client) -> Result<Vec<PersistentVolume>> {
        Ok(Vec::new())
    }
}

pub mod client_k8s_persistent_volume_claim {
    use super::*;
    use crate::core::client::kube_resources::PersistentVolumeClaim;

    pub async fn fetch_persistent_volume_claims(_token: &str, _client: &reqwest::Client) -> Result<Vec<PersistentVolumeClaim>> {
        Ok(Vec::new())
    }
}

pub mod client_k8s_persistent_volume_mapper {}
pub mod client_k8s_persistent_volume_claim_mapper {}

// ResourceQuota compatibility
pub mod client_k8s_resource_quota {
    use super::*;
    use crate::core::client::kube_resources::ResourceQuota;

    pub async fn fetch_resource_quotas(_token: &str, _client: &reqwest::Client) -> Result<Vec<ResourceQuota>> {
        Ok(Vec::new())
    }
}

pub mod client_k8s_resource_quota_mapper {}
