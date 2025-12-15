// Compatibility shim for old k8s client
// TODO: Migrate all services to use new kube-rs client directly

#![allow(dead_code, unused_variables)]

use anyhow::Result;

// Util module compatibility
pub mod util {
    use anyhow::Result;
    use reqwest::Client;
    use std::env;

    pub fn read_token() -> Result<String> {
        if let Ok(token) = std::env::var("KUBE_TOKEN") {
            return Ok(token);
        }

        if let Ok(path) = std::env::var("RUSTCOST_TOKEN_PATH") {
            if let Ok(token) = std::fs::read_to_string(&path) {
                return Ok(token.trim().to_string());
            }
        }

        match std::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/token") {
            Ok(token) => Ok(token.trim().to_string()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn build_client() -> Result<Client> {
        Ok(Client::new())
    }

    pub fn k8s_api_server() -> String {
        env::var("RUSTCOST_K8S_API_URL")
            .unwrap_or_else(|_| "https://kubernetes.default.svc".to_string())
    }

    pub async fn build_kube_client() -> Result<kube::Client> {
        crate::core::client::kube_client::build_kube_client().await
    }
}

// Pod client compatibility
pub mod client_k8s_pod {
    use super::*;
    use crate::core::client::{kube_client, pods};
    use crate::core::client::kube_resources::Pod;
    use kube::api::{DeleteParams, Patch, PatchParams, PostParams};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct PodList {
        pub items: Vec<Pod>,
    }

    /// Fetch ALL pods using kube-rs
    pub async fn fetch_pods(_token: &str, _client: &reqwest::Client) -> Result<PodList> {
        let kube = kube_client::build_kube_client().await?;
        let items = pods::fetch_pods(&kube).await?;
        Ok(PodList { items })
    }

    pub async fn fetch_pod_by_uid(_token: &str, _client: &reqwest::Client, uid: &str) -> Result<Pod> {
        let kube = kube_client::build_kube_client().await?;
        pods::fetch_pod_by_uid(&kube, uid).await
    }
    pub async fn fetch_pod_by_name_and_namespace(
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<Pod> {
        let kube = kube_client::build_kube_client().await?;
        pods::fetch_pod_by_name_and_namespace(&kube, namespace, name).await
    }

    pub async fn fetch_pods_by_label(
        _token: &str,
        _client: &reqwest::Client,
        label: &str,
    ) -> Result<PodList> {
        let kube = kube_client::build_kube_client().await?;
        let items = pods::fetch_pods_by_label(&kube, label).await?;
        Ok(PodList { items })
    }

    pub async fn fetch_pods_by_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
    ) -> Result<PodList> {
        let kube = kube_client::build_kube_client().await?;
        let items = pods::fetch_pods_by_namespace(&kube, namespace).await?;
        Ok(PodList { items })
    }

    pub async fn fetch_pods_by_node(
        _token: &str,
        _client: &reqwest::Client,
        node: &str,
    ) -> Result<PodList> {
        let kube = kube_client::build_kube_client().await?;
        let items = pods::fetch_pods_by_node(&kube, node).await?;
        Ok(PodList { items })
    }

    pub async fn create_pod(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        pod: &Pod,
    ) -> Result<Pod> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Pod> = kube::Api::namespaced(kube, namespace);
        Ok(api.create(&PostParams::default(), pod).await?)
    }

    pub async fn patch_pod(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
        patch: Value,
    ) -> Result<Pod> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Pod> = kube::Api::namespaced(kube, namespace);
        Ok(api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await?)
    }

    pub async fn delete_pod(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<()> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Pod> = kube::Api::namespaced(kube, namespace);
        let _ = api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }
}

// Pod mapper compatibility
pub mod client_k8s_pod_mapper {
    use super::*;
    use crate::core::client::kube_resources::Pod;
    use crate::core::persistence::info::k8s::pod::info_pod_entity::InfoPodEntity;

    pub fn map_pod_to_info_pod_entity(pod: &Pod) -> Result<InfoPodEntity> {
        crate::core::client::mappers::map_pod_to_info_entity(pod)
    }
}

// Node client compatibility
pub mod client_k8s_node {
    use super::*;
    use crate::core::client::{kube_client, nodes};
    use crate::core::client::kube_resources::Node;
    use kube::api::ListParams;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone)]
    pub struct NodeList {
        pub items: Vec<Node>,
    }

    /// Fetch all nodes
    pub async fn fetch_nodes(_token: &str, _client: &reqwest::Client) -> Result<NodeList> {
        let kube = kube_client::build_kube_client().await?;
        let items = nodes::fetch_nodes(&kube).await?;
        Ok(NodeList { items })
    }

    /// Fetch a node by name
    pub async fn fetch_node_by_name(
        _token: &str,
        _client: &reqwest::Client,
        name: &str,
    ) -> Result<Node> {
        let kube = kube_client::build_kube_client().await?;
        nodes::fetch_node_by_name(&kube, name).await
    }

    /// Fetch nodes by label selector (e.g., "node-role.kubernetes.io/worker=")
    pub async fn fetch_nodes_by_label(
        _token: &str,
        _client: &reqwest::Client,
        label_selector: &str,
    ) -> Result<NodeList> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Node> = kube::Api::all(kube.clone());
        let lp = ListParams::default().labels(label_selector);
        let list = api.list(&lp).await?;
        Ok(NodeList { items: list.items })
    }
}

// Deployment compatibility
pub mod client_k8s_deployment {
    use super::*;
    use crate::core::client::{deployments, kube_client};
    use crate::core::client::kube_resources::Deployment;
    use kube::api::{DeleteParams, Patch, PatchParams, PostParams};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct DeploymentList {
        pub items: Vec<Deployment>,
    }

    pub async fn fetch_deployments(
        _token: &str,
        _client: &reqwest::Client,
    ) -> Result<DeploymentList> {
        let kube = kube_client::build_kube_client().await?;
        let items = deployments::fetch_deployments(&kube).await?;
        Ok(DeploymentList { items })
    }

    pub async fn fetch_deployment_by_name_and_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<Deployment> {
        let kube = kube_client::build_kube_client().await?;
        deployments::fetch_deployment_by_name_and_namespace(&kube, namespace, name).await
    }

    pub async fn fetch_deployments_by_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
    ) -> Result<DeploymentList> {
        let kube = kube_client::build_kube_client().await?;
        let items = deployments::fetch_deployments_by_namespace(&kube, namespace).await?;
        Ok(DeploymentList { items })
    }

    pub async fn fetch_deployments_by_label(
        _token: &str,
        _client: &reqwest::Client,
        label: &str,
    ) -> Result<DeploymentList> {
        let kube = kube_client::build_kube_client().await?;
        let items = deployments::fetch_deployments_by_label(&kube, label).await?;
        Ok(DeploymentList { items })
    }

    pub async fn create_deployment(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        deployment: &Deployment,
    ) -> Result<Deployment> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Deployment> = kube::Api::namespaced(kube, namespace);
        Ok(api.create(&PostParams::default(), deployment).await?)
    }

    pub async fn patch_deployment(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
        patch: Value,
    ) -> Result<Deployment> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Deployment> = kube::Api::namespaced(kube, namespace);
        Ok(api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await?)
    }

    pub async fn delete_deployment(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<()> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Deployment> = kube::Api::namespaced(kube, namespace);
        let _ = api.delete(name, &DeleteParams::default()).await?;
        Ok(())
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

// StatefulSet compatibility
pub mod client_k8s_statefulset {
    use super::*;
    use crate::core::client::{kube_client, statefulsets};
    use crate::core::client::kube_resources::StatefulSet;
    use kube::api::{DeleteParams, Patch, PatchParams, PostParams};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct StatefulSetList {
        pub items: Vec<StatefulSet>,
    }

    pub async fn fetch_statefulsets(
        _token: &str,
        _client: &reqwest::Client,
    ) -> Result<StatefulSetList> {
        let kube = kube_client::build_kube_client().await?;
        let items = statefulsets::fetch_statefulsets(&kube).await?;
        Ok(StatefulSetList { items })
    }

    pub async fn fetch_statefulset_by_name_and_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<StatefulSet> {
        let kube = kube_client::build_kube_client().await?;
        statefulsets::fetch_statefulset_by_name_and_namespace(&kube, namespace, name).await
    }

    pub async fn fetch_statefulsets_by_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
    ) -> Result<StatefulSetList> {
        let kube = kube_client::build_kube_client().await?;
        let items = statefulsets::fetch_statefulsets_by_namespace(&kube, namespace).await?;
        Ok(StatefulSetList { items })
    }

    pub async fn fetch_statefulsets_by_label(
        _token: &str,
        _client: &reqwest::Client,
        label: &str,
    ) -> Result<StatefulSetList> {
        let kube = kube_client::build_kube_client().await?;
        let items = statefulsets::fetch_statefulsets_by_label(&kube, label).await?;
        Ok(StatefulSetList { items })
    }

    pub async fn create_statefulset(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        statefulset: &StatefulSet,
    ) -> Result<StatefulSet> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<StatefulSet> = kube::Api::namespaced(kube, namespace);
        Ok(api.create(&PostParams::default(), statefulset).await?)
    }

    pub async fn patch_statefulset(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
        patch: Value,
    ) -> Result<StatefulSet> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<StatefulSet> = kube::Api::namespaced(kube, namespace);
        Ok(api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await?)
    }

    pub async fn delete_statefulset(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<()> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<StatefulSet> = kube::Api::namespaced(kube, namespace);
        let _ = api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }
}

// DaemonSet compatibility
pub mod client_k8s_daemonset {
    use super::*;
    use crate::core::client::{daemonsets, kube_client};
    use crate::core::client::kube_resources::DaemonSet;
    use kube::api::{DeleteParams, Patch, PatchParams, PostParams};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct DaemonSetList {
        pub items: Vec<DaemonSet>,
    }

    pub async fn fetch_daemonsets(
        _token: &str,
        _client: &reqwest::Client,
    ) -> Result<DaemonSetList> {
        let kube = kube_client::build_kube_client().await?;
        let items = daemonsets::fetch_daemonsets(&kube).await?;
        Ok(DaemonSetList { items })
    }

    pub async fn fetch_daemonset_by_name_and_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<DaemonSet> {
        let kube = kube_client::build_kube_client().await?;
        daemonsets::fetch_daemonset_by_name_and_namespace(&kube, namespace, name).await
    }

    pub async fn fetch_daemonsets_by_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
    ) -> Result<DaemonSetList> {
        let kube = kube_client::build_kube_client().await?;
        let items = daemonsets::fetch_daemonsets_by_namespace(&kube, namespace).await?;
        Ok(DaemonSetList { items })
    }

    pub async fn fetch_daemonsets_by_label(
        _token: &str,
        _client: &reqwest::Client,
        label: &str,
    ) -> Result<DaemonSetList> {
        let kube = kube_client::build_kube_client().await?;
        let items = daemonsets::fetch_daemonsets_by_label(&kube, label).await?;
        Ok(DaemonSetList { items })
    }

    pub async fn create_daemonset(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        daemonset: &DaemonSet,
    ) -> Result<DaemonSet> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<DaemonSet> = kube::Api::namespaced(kube, namespace);
        Ok(api.create(&PostParams::default(), daemonset).await?)
    }

    pub async fn patch_daemonset(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
        patch: Value,
    ) -> Result<DaemonSet> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<DaemonSet> = kube::Api::namespaced(kube, namespace);
        Ok(api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await?)
    }

    pub async fn delete_daemonset(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<()> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<DaemonSet> = kube::Api::namespaced(kube, namespace);
        let _ = api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }
}

// Job compatibility
pub mod client_k8s_job {
    use super::*;
    use crate::core::client::{jobs, kube_client};
    use crate::core::client::kube_resources::Job;
    use kube::api::{DeleteParams, Patch, PatchParams, PostParams};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct JobList {
        pub items: Vec<Job>,
    }

    pub async fn fetch_jobs(_token: &str, _client: &reqwest::Client) -> Result<JobList> {
        let kube = kube_client::build_kube_client().await?;
        let items = jobs::fetch_jobs(&kube).await?;
        Ok(JobList { items })
    }

    pub async fn fetch_job_by_name_and_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<Job> {
        let kube = kube_client::build_kube_client().await?;
        jobs::fetch_job_by_name_and_namespace(&kube, namespace, name).await
    }

    pub async fn fetch_jobs_by_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
    ) -> Result<JobList> {
        let kube = kube_client::build_kube_client().await?;
        let items = jobs::fetch_jobs_by_namespace(&kube, namespace).await?;
        Ok(JobList { items })
    }

    pub async fn fetch_jobs_by_label(
        _token: &str,
        _client: &reqwest::Client,
        label: &str,
    ) -> Result<JobList> {
        let kube = kube_client::build_kube_client().await?;
        let items = jobs::fetch_jobs_by_label(&kube, label).await?;
        Ok(JobList { items })
    }

    pub async fn create_job(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        job: &Job,
    ) -> Result<Job> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Job> = kube::Api::namespaced(kube, namespace);
        Ok(api.create(&PostParams::default(), job).await?)
    }

    pub async fn patch_job(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
        patch: Value,
    ) -> Result<Job> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Job> = kube::Api::namespaced(kube, namespace);
        Ok(api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await?)
    }

    pub async fn delete_job(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<()> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Job> = kube::Api::namespaced(kube, namespace);
        let _ = api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }
}

// CronJob compatibility
pub mod client_k8s_cronjob {
    use super::*;
    use crate::core::client::{cronjobs, kube_client};
    use crate::core::client::kube_resources::CronJob;
    use kube::api::{DeleteParams, Patch, PatchParams, PostParams};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct CronJobList {
        pub items: Vec<CronJob>,
    }

    pub async fn fetch_cronjobs(
        _token: &str,
        _client: &reqwest::Client,
    ) -> Result<CronJobList> {
        let kube = kube_client::build_kube_client().await?;
        let items = cronjobs::fetch_cronjobs(&kube).await?;
        Ok(CronJobList { items })
    }

    pub async fn fetch_cronjob_by_name_and_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<CronJob> {
        let kube = kube_client::build_kube_client().await?;
        cronjobs::fetch_cronjob_by_name_and_namespace(&kube, namespace, name).await
    }

    pub async fn fetch_cronjobs_by_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
    ) -> Result<CronJobList> {
        let kube = kube_client::build_kube_client().await?;
        let items = cronjobs::fetch_cronjobs_by_namespace(&kube, namespace).await?;
        Ok(CronJobList { items })
    }

    pub async fn fetch_cronjobs_by_label(
        _token: &str,
        _client: &reqwest::Client,
        label: &str,
    ) -> Result<CronJobList> {
        let kube = kube_client::build_kube_client().await?;
        let items = cronjobs::fetch_cronjobs_by_label(&kube, label).await?;
        Ok(CronJobList { items })
    }

    pub async fn create_cronjob(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        cronjob: &CronJob,
    ) -> Result<CronJob> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<CronJob> = kube::Api::namespaced(kube, namespace);
        Ok(api.create(&PostParams::default(), cronjob).await?)
    }

    pub async fn patch_cronjob(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
        patch: Value,
    ) -> Result<CronJob> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<CronJob> = kube::Api::namespaced(kube, namespace);
        Ok(api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await?)
    }

    pub async fn delete_cronjob(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<()> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<CronJob> = kube::Api::namespaced(kube, namespace);
        let _ = api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }
}

// Service compatibility
pub mod client_k8s_service {
    use super::*;
    use crate::core::client::{kube_client, services};
    use crate::core::client::kube_resources::Service;
    use kube::api::{DeleteParams, Patch, PatchParams, PostParams};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct ServiceList {
        pub items: Vec<Service>,
    }

    pub async fn fetch_services(
        _token: &str,
        _client: &reqwest::Client,
    ) -> Result<ServiceList> {
        let kube = kube_client::build_kube_client().await?;
        let items = services::fetch_services(&kube).await?;
        Ok(ServiceList { items })
    }

    pub async fn fetch_service_by_name_and_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<Service> {
        let kube = kube_client::build_kube_client().await?;
        services::fetch_service_by_name_and_namespace(&kube, namespace, name).await
    }

    pub async fn fetch_services_by_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
    ) -> Result<ServiceList> {
        let kube = kube_client::build_kube_client().await?;
        let items = services::fetch_services_by_namespace(&kube, namespace).await?;
        Ok(ServiceList { items })
    }

    pub async fn fetch_services_by_label(
        _token: &str,
        _client: &reqwest::Client,
        label: &str,
    ) -> Result<ServiceList> {
        let kube = kube_client::build_kube_client().await?;
        let items = services::fetch_services_by_label(&kube, label).await?;
        Ok(ServiceList { items })
    }

    pub async fn create_service(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        service: &Service,
    ) -> Result<Service> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Service> = kube::Api::namespaced(kube, namespace);
        Ok(api.create(&PostParams::default(), service).await?)
    }

    pub async fn patch_service(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
        patch: Value,
    ) -> Result<Service> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Service> = kube::Api::namespaced(kube, namespace);
        Ok(api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await?)
    }

    pub async fn delete_service(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<()> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Service> = kube::Api::namespaced(kube, namespace);
        let _ = api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }
}

// Ingress compatibility
pub mod client_k8s_ingress {
    use super::*;
    use crate::core::client::{ingresses, kube_client};
    use crate::core::client::kube_resources::Ingress;
    use kube::api::{DeleteParams, Patch, PatchParams, PostParams};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct IngressList {
        pub items: Vec<Ingress>,
    }

    pub async fn fetch_ingresses(
        _token: &str,
        _client: &reqwest::Client,
    ) -> Result<IngressList> {
        let kube = kube_client::build_kube_client().await?;
        let items = ingresses::fetch_ingresses(&kube).await?;
        Ok(IngressList { items })
    }

    pub async fn fetch_ingress_by_name_and_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<Ingress> {
        let kube = kube_client::build_kube_client().await?;
        ingresses::fetch_ingress_by_name_and_namespace(&kube, namespace, name).await
    }

    pub async fn fetch_ingresses_by_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
    ) -> Result<IngressList> {
        let kube = kube_client::build_kube_client().await?;
        let items = ingresses::fetch_ingresses_by_namespace(&kube, namespace).await?;
        Ok(IngressList { items })
    }

    pub async fn fetch_ingresses_by_label(
        _token: &str,
        _client: &reqwest::Client,
        label: &str,
    ) -> Result<IngressList> {
        let kube = kube_client::build_kube_client().await?;
        let items = ingresses::fetch_ingresses_by_label(&kube, label).await?;
        Ok(IngressList { items })
    }

    pub async fn create_ingress(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        ingress: &Ingress,
    ) -> Result<Ingress> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Ingress> = kube::Api::namespaced(kube, namespace);
        Ok(api.create(&PostParams::default(), ingress).await?)
    }

    pub async fn patch_ingress(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
        patch: Value,
    ) -> Result<Ingress> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Ingress> = kube::Api::namespaced(kube, namespace);
        Ok(api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await?)
    }

    pub async fn delete_ingress(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<()> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Ingress> = kube::Api::namespaced(kube, namespace);
        let _ = api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }
}

// Namespace compatibility
pub mod client_k8s_namespace {
    use super::*;
    use crate::core::client::kube_resources::Namespace;
    use crate::core::client::{kube_client, namespaces};
    use kube::api::{DeleteParams, Patch, PatchParams, PostParams};
    use serde::{Serialize, Deserialize};
    use serde_json::Value;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct NamespaceList {
        pub items: Vec<Namespace>,
    }

    pub async fn fetch_namespaces(_token: &str, _client: &reqwest::Client) -> Result<NamespaceList> {
        let kube = kube_client::build_kube_client().await?;
        let items = namespaces::fetch_namespaces(&kube).await?;
        Ok(NamespaceList { items })
    }

    pub async fn fetch_namespace_by_name(
        _token: &str,
        _client: &reqwest::Client,
        name: &str,
    ) -> Result<Namespace> {
        let kube = kube_client::build_kube_client().await?;
        namespaces::fetch_namespace_by_name(&kube, name).await
    }

    pub async fn create_namespace(
        _token: &str,
        _client: &reqwest::Client,
        ns: &Namespace,
    ) -> Result<Namespace> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Namespace> = kube::Api::all(kube);
        Ok(api.create(&PostParams::default(), ns).await?)
    }

    pub async fn patch_namespace(
        _token: &str,
        _client: &reqwest::Client,
        name: &str,
        patch: Value,
    ) -> Result<Namespace> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Namespace> = kube::Api::all(kube);
        Ok(api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await?)
    }

    pub async fn delete_namespace(
        _token: &str,
        _client: &reqwest::Client,
        name: &str,
    ) -> Result<()> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<Namespace> = kube::Api::all(kube);
        let _ = api.delete(name, &DeleteParams::default()).await?;
        Ok(())
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
    use crate::core::client::{kube_client, other_resources};
    use crate::core::client::kube_resources::HorizontalPodAutoscaler;
    use kube::api::{DeleteParams, Patch, PatchParams, PostParams};
    use serde_json::Value;

    pub async fn fetch_hpas(_token: &str, _client: &reqwest::Client) -> Result<Vec<HorizontalPodAutoscaler>> {
        let kube = kube_client::build_kube_client().await?;
        other_resources::fetch_hpas(&kube).await
    }

    pub async fn fetch_horizontal_pod_autoscalers(_token: &str, _client: &reqwest::Client) -> Result<Vec<HorizontalPodAutoscaler>> {
        fetch_hpas(_token, _client).await
    }

    pub async fn create_hpa(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        hpa: &HorizontalPodAutoscaler,
    ) -> Result<HorizontalPodAutoscaler> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<HorizontalPodAutoscaler> = kube::Api::namespaced(kube, namespace);
        Ok(api.create(&PostParams::default(), hpa).await?)
    }

    pub async fn patch_hpa(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
        patch: Value,
    ) -> Result<HorizontalPodAutoscaler> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<HorizontalPodAutoscaler> = kube::Api::namespaced(kube, namespace);
        Ok(api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await?)
    }

    pub async fn delete_hpa(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<()> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<HorizontalPodAutoscaler> = kube::Api::namespaced(kube, namespace);
        let _ = api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }
}

pub mod client_k8s_hpa_mapper {}

// LimitRange compatibility
pub mod client_k8s_limit_range {
    use super::*;
    use crate::core::client::{kube_client, other_resources};
    use crate::core::client::kube_resources::LimitRange;
    use kube::api::{DeleteParams, Patch, PatchParams, PostParams};
    use serde_json::Value;

    pub async fn fetch_limit_ranges(_token: &str, _client: &reqwest::Client) -> Result<Vec<LimitRange>> {
        let kube = kube_client::build_kube_client().await?;
        other_resources::fetch_limit_ranges(&kube).await
    }

    pub async fn create_limit_range(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        lr: &LimitRange,
    ) -> Result<LimitRange> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<LimitRange> = kube::Api::namespaced(kube, namespace);
        Ok(api.create(&PostParams::default(), lr).await?)
    }

    pub async fn patch_limit_range(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
        patch: Value,
    ) -> Result<LimitRange> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<LimitRange> = kube::Api::namespaced(kube, namespace);
        Ok(api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await?)
    }

    pub async fn delete_limit_range(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<()> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<LimitRange> = kube::Api::namespaced(kube, namespace);
        let _ = api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }
}

pub mod client_k8s_limit_range_mapper {}

// PV/PVC compatibility
pub mod client_k8s_persistent_volume {
    use super::*;
    use crate::core::client::{kube_client, other_resources};
    use crate::core::client::kube_resources::PersistentVolume;
    use kube::api::{DeleteParams, Patch, PatchParams, PostParams};
    use serde_json::Value;

    pub async fn fetch_persistent_volumes(_token: &str, _client: &reqwest::Client) -> Result<Vec<PersistentVolume>> {
        let kube = kube_client::build_kube_client().await?;
        other_resources::fetch_persistent_volumes(&kube).await
    }

    pub async fn fetch_persistent_volume_by_name(
        _token: &str,
        _client: &reqwest::Client,
        name: &str,
    ) -> Result<PersistentVolume> {
        let kube = kube_client::build_kube_client().await?;
        other_resources::fetch_persistent_volume_by_name(&kube, name).await
    }

    pub async fn create_persistent_volume(
        _token: &str,
        _client: &reqwest::Client,
        pv: &PersistentVolume,
    ) -> Result<PersistentVolume> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<PersistentVolume> = kube::Api::all(kube);
        Ok(api.create(&PostParams::default(), pv).await?)
    }

    pub async fn patch_persistent_volume(
        _token: &str,
        _client: &reqwest::Client,
        name: &str,
        patch: Value,
    ) -> Result<PersistentVolume> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<PersistentVolume> = kube::Api::all(kube);
        Ok(api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await?)
    }

    pub async fn delete_persistent_volume(
        _token: &str,
        _client: &reqwest::Client,
        name: &str,
    ) -> Result<()> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<PersistentVolume> = kube::Api::all(kube);
        let _ = api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }
}

pub mod client_k8s_persistent_volume_claim {
    use super::*;
    use crate::core::client::{kube_client, other_resources};
    use crate::core::client::kube_resources::PersistentVolumeClaim;
    use kube::api::{DeleteParams, Patch, PatchParams, PostParams};
    use serde_json::Value;

    pub async fn fetch_persistent_volume_claims(_token: &str, _client: &reqwest::Client) -> Result<Vec<PersistentVolumeClaim>> {
        let kube = kube_client::build_kube_client().await?;
        other_resources::fetch_persistent_volume_claims(&kube).await
    }

    pub async fn fetch_persistent_volume_claim_by_name_and_namespace(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<PersistentVolumeClaim> {
        let kube = kube_client::build_kube_client().await?;
        other_resources::fetch_persistent_volume_claim_by_name_and_namespace(&kube, namespace, name).await
    }

    pub async fn create_persistent_volume_claim(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        pvc: &PersistentVolumeClaim,
    ) -> Result<PersistentVolumeClaim> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<PersistentVolumeClaim> = kube::Api::namespaced(kube, namespace);
        Ok(api.create(&PostParams::default(), pvc).await?)
    }

    pub async fn patch_persistent_volume_claim(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
        patch: Value,
    ) -> Result<PersistentVolumeClaim> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<PersistentVolumeClaim> = kube::Api::namespaced(kube, namespace);
        Ok(api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await?)
    }

    pub async fn delete_persistent_volume_claim(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<()> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<PersistentVolumeClaim> = kube::Api::namespaced(kube, namespace);
        let _ = api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }
}

pub mod client_k8s_persistent_volume_mapper {}
pub mod client_k8s_persistent_volume_claim_mapper {}

// ResourceQuota compatibility
pub mod client_k8s_resource_quota {
    use super::*;
    use crate::core::client::{kube_client, other_resources};
    use crate::core::client::kube_resources::ResourceQuota;
    use kube::api::{DeleteParams, Patch, PatchParams, PostParams};
    use serde_json::Value;

    pub async fn fetch_resource_quotas(_token: &str, _client: &reqwest::Client) -> Result<Vec<ResourceQuota>> {
        let kube = kube_client::build_kube_client().await?;
        other_resources::fetch_resource_quotas(&kube).await
    }

    pub async fn create_resource_quota(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        rq: &ResourceQuota,
    ) -> Result<ResourceQuota> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<ResourceQuota> = kube::Api::namespaced(kube, namespace);
        Ok(api.create(&PostParams::default(), rq).await?)
    }

    pub async fn patch_resource_quota(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
        patch: Value,
    ) -> Result<ResourceQuota> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<ResourceQuota> = kube::Api::namespaced(kube, namespace);
        Ok(api
            .patch(name, &PatchParams::default(), &Patch::Merge(patch))
            .await?)
    }

    pub async fn delete_resource_quota(
        _token: &str,
        _client: &reqwest::Client,
        namespace: &str,
        name: &str,
    ) -> Result<()> {
        let kube = kube_client::build_kube_client().await?;
        let api: kube::Api<ResourceQuota> = kube::Api::namespaced(kube, namespace);
        let _ = api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }
}

pub mod client_k8s_resource_quota_mapper {}
