use anyhow::Result;
use kube::{Api, Client};
use kube::api::ListParams;
use tracing::debug;

use crate::core::client::kube_resources::{
    PersistentVolume, PersistentVolumeClaim, ResourceQuota, LimitRange, HorizontalPodAutoscaler,
};

// ==================== Persistent Volumes ====================

/// Fetch all persistent volumes in the cluster
pub async fn fetch_persistent_volumes(client: &Client) -> Result<Vec<PersistentVolume>> {
    let pvs: Api<PersistentVolume> = Api::all(client.clone());
    let pv_list = pvs.list(&ListParams::default()).await?;

    debug!("Discovered {} persistent volume(s)", pv_list.items.len());
    Ok(pv_list.items)
}

/// Fetch a single persistent volume by name
pub async fn fetch_persistent_volume_by_name(
    client: &Client,
    name: &str,
) -> Result<PersistentVolume> {
    let pvs: Api<PersistentVolume> = Api::all(client.clone());
    let pv = pvs.get(name).await?;

    debug!("Fetched persistent volume: {}", name);
    Ok(pv)
}

// ==================== Persistent Volume Claims ====================

/// Fetch all persistent volume claims in the cluster
pub async fn fetch_persistent_volume_claims(client: &Client) -> Result<Vec<PersistentVolumeClaim>> {
    let pvcs: Api<PersistentVolumeClaim> = Api::all(client.clone());
    let pvc_list = pvcs.list(&ListParams::default()).await?;

    debug!("Discovered {} persistent volume claim(s)", pvc_list.items.len());
    Ok(pvc_list.items)
}

/// Fetch persistent volume claims in a specific namespace
pub async fn fetch_persistent_volume_claims_by_namespace(
    client: &Client,
    namespace: &str,
) -> Result<Vec<PersistentVolumeClaim>> {
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
    let pvc_list = pvcs.list(&ListParams::default()).await?;

    debug!(
        "Discovered {} persistent volume claim(s) in namespace '{}'",
        pvc_list.items.len(),
        namespace
    );
    Ok(pvc_list.items)
}

/// Fetch a single persistent volume claim by name and namespace
pub async fn fetch_persistent_volume_claim_by_name_and_namespace(
    client: &Client,
    namespace: &str,
    name: &str,
) -> Result<PersistentVolumeClaim> {
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
    let pvc = pvcs.get(name).await?;

    debug!("Fetched persistent volume claim: {}/{}", namespace, name);
    Ok(pvc)
}

// ==================== Resource Quotas ====================

/// Fetch all resource quotas in the cluster
pub async fn fetch_resource_quotas(client: &Client) -> Result<Vec<ResourceQuota>> {
    let quotas: Api<ResourceQuota> = Api::all(client.clone());
    let quota_list = quotas.list(&ListParams::default()).await?;

    debug!("Discovered {} resource quota(s)", quota_list.items.len());
    Ok(quota_list.items)
}

/// Fetch resource quotas in a specific namespace
pub async fn fetch_resource_quotas_by_namespace(
    client: &Client,
    namespace: &str,
) -> Result<Vec<ResourceQuota>> {
    let quotas: Api<ResourceQuota> = Api::namespaced(client.clone(), namespace);
    let quota_list = quotas.list(&ListParams::default()).await?;

    debug!(
        "Discovered {} resource quota(s) in namespace '{}'",
        quota_list.items.len(),
        namespace
    );
    Ok(quota_list.items)
}

// ==================== Limit Ranges ====================

/// Fetch all limit ranges in the cluster
pub async fn fetch_limit_ranges(client: &Client) -> Result<Vec<LimitRange>> {
    let limits: Api<LimitRange> = Api::all(client.clone());
    let limit_list = limits.list(&ListParams::default()).await?;

    debug!("Discovered {} limit range(s)", limit_list.items.len());
    Ok(limit_list.items)
}

/// Fetch limit ranges in a specific namespace
pub async fn fetch_limit_ranges_by_namespace(
    client: &Client,
    namespace: &str,
) -> Result<Vec<LimitRange>> {
    let limits: Api<LimitRange> = Api::namespaced(client.clone(), namespace);
    let limit_list = limits.list(&ListParams::default()).await?;

    debug!(
        "Discovered {} limit range(s) in namespace '{}'",
        limit_list.items.len(),
        namespace
    );
    Ok(limit_list.items)
}

// ==================== Horizontal Pod Autoscalers ====================

/// Fetch all HPAs in the cluster
pub async fn fetch_hpas(client: &Client) -> Result<Vec<HorizontalPodAutoscaler>> {
    let hpas: Api<HorizontalPodAutoscaler> = Api::all(client.clone());
    let hpa_list = hpas.list(&ListParams::default()).await?;

    debug!("Discovered {} HPA(s)", hpa_list.items.len());
    Ok(hpa_list.items)
}

/// Fetch HPAs in a specific namespace
pub async fn fetch_hpas_by_namespace(
    client: &Client,
    namespace: &str,
) -> Result<Vec<HorizontalPodAutoscaler>> {
    let hpas: Api<HorizontalPodAutoscaler> = Api::namespaced(client.clone(), namespace);
    let hpa_list = hpas.list(&ListParams::default()).await?;

    debug!(
        "Discovered {} HPA(s) in namespace '{}'",
        hpa_list.items.len(),
        namespace
    );
    Ok(hpa_list.items)
}
