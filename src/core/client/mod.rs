// Compatibility shim for old k8s client (temporary)
pub mod k8s {
    pub use super::k8s_compat::*;
}
pub mod k8s_compat;

// Kube-rs based Kubernetes client
pub mod kube_client;
pub mod kube_resources;
pub mod nodes;
pub mod pods;
pub mod deployments;
pub mod statefulsets;
pub mod daemonsets;
pub mod jobs;
pub mod cronjobs;
pub mod services;
pub mod ingresses;
pub mod namespaces;
pub mod other_resources;
pub mod watchers;
pub mod store;
pub mod mappers;

// Other clients
pub mod llm_client;
pub mod slack_client;
