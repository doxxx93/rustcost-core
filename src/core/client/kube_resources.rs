/// Re-export commonly used Kubernetes resource types from k8s-openapi
/// This module provides a centralized place for all K8s resource types

pub use k8s_openapi::api::core::v1::{
    Container as K8sContainer,
    ContainerStatus,
    Node,
    Pod,
    PodSpec,
    PodStatus,
    Namespace,
    PersistentVolume,
    PersistentVolumeClaim,
    Service,
    ResourceQuota,
    LimitRange,
};

pub use k8s_openapi::api::apps::v1::{
    Deployment,
    ReplicaSet,
    StatefulSet,
    DaemonSet,
};

pub use k8s_openapi::api::batch::v1::{
    Job,
    CronJob,
};

pub use k8s_openapi::api::networking::v1::{
    Ingress,
};

pub use k8s_openapi::api::autoscaling::v2::{
    HorizontalPodAutoscaler,
};

pub use k8s_openapi::apimachinery::pkg::apis::meta::v1::{
    ObjectMeta,
    ListMeta,
    OwnerReference,
};
