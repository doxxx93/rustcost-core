use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::{collections::{HashMap, HashSet}, fs};

use crate::api::dto::metrics_dto::RangeQuery;
use crate::core::persistence::info::{
    k8s::pod::{info_pod_entity::InfoPodEntity, info_pod_repository::InfoPodRepository},
    path::info_k8s_pod_dir_path,
};
use crate::core::persistence::info::k8s::pod::info_pod_api_repository_trait::InfoPodApiRepository;
use crate::domain::metric::k8s::common::dto::{
    MetricGetResponseDto, MetricScope, MetricSeriesDto, UniversalMetricPointDto,
};
use crate::domain::metric::k8s::common::service_helpers::{
    apply_costs, build_cost_summary_dto, build_cost_trend_dto, build_raw_summary_value,
};
use crate::domain::metric::k8s::namespace::service::aggregate_namespace_points;

use crate::domain::info::service::info_unit_price_service;
use crate::domain::metric::k8s::pod::service::build_pod_response_from_infos;

// ------------------------------
// Helpers
// ------------------------------

/// Load pods grouped by deployment name from local pod info.
fn load_pods_by_deployment(filter: &[String]) -> Result<HashMap<String, Vec<InfoPodEntity>>> {
    let mut map = HashMap::new();
    let dir = info_k8s_pod_dir_path();

    if !dir.exists() {
        return Ok(map);
    }

    let filters: HashSet<String> = filter.iter().cloned().collect();
    let allow_all = filters.is_empty();
    let repo = InfoPodRepository::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let pod_uid = entry.file_name().to_string_lossy().to_string();

        if let Ok(pod) = repo.read(&pod_uid) {
            if let Some(owner) = pod.owner_name.clone() {
                if allow_all || filters.contains(&owner) {
                    map.entry(owner).or_default().push(pod);
                }
            }
        }
    }

    Ok(map)
}

fn pods_for_deployment(depl: &str) -> Result<Vec<InfoPodEntity>> {
    let map = load_pods_by_deployment(&[depl.to_string()])?;

    if let Some(pods) = map.get(depl) {
        if !pods.is_empty() {
            return Ok(pods.clone());
        }
    }

    Err(anyhow!("deployment '{}' has no pods", depl))
}

fn all_pods_for(deployments: &[String]) -> Result<Vec<InfoPodEntity>> {
    let map = load_pods_by_deployment(deployments)?;
    Ok(map.into_values().flatten().collect())
}

fn collect_targets(
    deployments: Vec<String>,
    map: &HashMap<String, Vec<InfoPodEntity>>,
) -> Vec<String> {
    if deployments.is_empty() {
        map.keys().cloned().collect::<Vec<_>>()
    } else {
        deployments
    }
}

fn aggregate_deployment_response(
    deployment: &str,
    per_pod_response: &MetricGetResponseDto,
) -> MetricGetResponseDto {
    let all_points: Vec<UniversalMetricPointDto> =
        per_pod_response.series.iter().flat_map(|s| s.points.clone()).collect();

    let aggregated_points = aggregate_namespace_points(all_points);

    MetricGetResponseDto {
        start: per_pod_response.start,
        end: per_pod_response.end,
        scope: "deployment".to_string(),
        target: Some(deployment.to_string()),
        granularity: per_pod_response.granularity.clone(),
        series: vec![MetricSeriesDto {
            key: deployment.to_string(),
            name: deployment.to_string(),
            scope: MetricScope::Deployment,
            points: aggregated_points,
            running_hours: None,
            cost_summary: None,
        }],
        total: None,
        limit: None,
        offset: None,
    }
}

// ------------------------------
// RAW (MULTIPLE)
// ------------------------------

pub async fn get_metric_k8s_deployments_raw(
    q: RangeQuery,
    deployments: Vec<String>,
) -> Result<Value> {
    let map = load_pods_by_deployment(&deployments)?;
    let target_list = collect_targets(deployments, &map);

    let mut series = Vec::new();
    let mut base = None;

    for depl in target_list {
        if let Some(pods) = map.get(&depl) {
            if pods.is_empty() {
                continue;
            }
            let pod_response = build_pod_response_from_infos(q.clone(), pods.clone(), Some(depl.clone()))?;
            let aggregated = aggregate_deployment_response(&depl, &pod_response);

            if base.is_none() {
                base = Some(aggregated.clone());
            }
            series.push(aggregated.series[0].clone());
        }
    }

    if let Some(mut final_resp) = base {
        final_resp.target = None;
        final_resp.series = series;
        return Ok(serde_json::to_value(final_resp)?);
    }

    Ok(json!({ "status": "no data" }))
}

// ------------------------------
// RAW (SINGLE)
// ------------------------------

pub async fn get_metric_k8s_deployment_raw(
    name: String,
    q: RangeQuery,
) -> Result<Value> {
    let pods = pods_for_deployment(&name)?;
    let pod_response = build_pod_response_from_infos(q, pods, Some(name.clone()))?;
    let aggregated = aggregate_deployment_response(&name, &pod_response);

    Ok(serde_json::to_value(aggregated)?)
}

// ------------------------------
// RAW SUMMARY (MULTIPLE)
// ------------------------------

pub async fn get_metric_k8s_deployments_raw_summary(
    q: RangeQuery,
    deployments: Vec<String>,
) -> Result<Value> {
    let map = load_pods_by_deployment(&deployments)?;
    let target_list = collect_targets(deployments, &map);

    let mut all_pods = Vec::new();
    for depl in target_list {
        if let Some(pods) = map.get(&depl) {
            all_pods.extend(pods.clone());
        }
    }

    if all_pods.is_empty() {
        return Ok(json!({ "status": "no data" }));
    }

    let per_pod = build_pod_response_from_infos(q, all_pods.clone(), None)?;
    let aggregated = aggregate_deployment_response("all", &per_pod);

    build_raw_summary_value(&aggregated, MetricScope::Deployment, all_pods.len())
}

// ------------------------------
// RAW SUMMARY (SINGLE)
// ------------------------------

pub async fn get_metric_k8s_deployment_raw_summary(
    name: String,
    q: RangeQuery,
) -> Result<Value> {
    let pods = pods_for_deployment(&name)?;
    let per_pod = build_pod_response_from_infos(q, pods.clone(), Some(name.clone()))?;
    let aggregated = aggregate_deployment_response(&name, &per_pod);

    build_raw_summary_value(&aggregated, MetricScope::Deployment, pods.len())
}

// ------------------------------
// RAW EFFICIENCY (NOT SUPPORTED)
// ------------------------------

pub async fn get_metric_k8s_deployments_raw_efficiency(
    _q: RangeQuery,
    _deployments: Vec<String>,
) -> Result<Value> {
    Ok(json!({
        "status": "not_supported",
        "message": "Deployment efficiency not supported yet"
    }))
}

pub async fn get_metric_k8s_deployment_raw_efficiency(
    _name: String,
    _q: RangeQuery,
) -> Result<Value> {
    Ok(json!({
        "status": "not_supported",
        "message": "Deployment efficiency not supported yet"
    }))
}

// ------------------------------
// COST (HELPERS)
// ------------------------------

async fn build_deployment_cost(
    deployment: Option<String>,
    q: RangeQuery,
    filter: &[String],
) -> Result<MetricGetResponseDto> {
    let pods = match deployment.as_ref() {
        Some(name) => pods_for_deployment(name)?,
        None => all_pods_for(filter)?,
    };

    if pods.is_empty() {
        return Err(anyhow!("no pods available for deployment cost calculation"));
    }

    let per_pod = build_pod_response_from_infos(q, pods, deployment.clone())?;
    Ok(aggregate_deployment_response(
        deployment.as_deref().unwrap_or("all"),
        &per_pod,
    ))
}

// ------------------------------
// COST (MULTIPLE)
// ------------------------------

pub async fn get_metric_k8s_deployments_cost(
    q: RangeQuery,
    deployments: Vec<String>,
) -> Result<Value> {
    let mut dto = build_deployment_cost(None, q, &deployments).await?;

    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    apply_costs(&mut dto, &unit_prices);

    Ok(serde_json::to_value(dto)?)
}

pub async fn get_metric_k8s_deployments_cost_summary(
    q: RangeQuery,
    deployments: Vec<String>,
) -> Result<Value> {
    let mut dto = build_deployment_cost(None, q, &deployments).await?;

    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    apply_costs(&mut dto, &unit_prices);

    let summary = build_cost_summary_dto(&dto, MetricScope::Deployment, None, &unit_prices);
    Ok(serde_json::to_value(summary)?)
}

pub async fn get_metric_k8s_deployments_cost_trend(
    q: RangeQuery,
    deployments: Vec<String>,
) -> Result<Value> {
    let mut dto = build_deployment_cost(None, q, &deployments).await?;

    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    apply_costs(&mut dto, &unit_prices);

    let trend = build_cost_trend_dto(&dto, MetricScope::Deployment, None)?;
    Ok(serde_json::to_value(trend)?)
}

// ------------------------------
// COST (SINGLE)
// ------------------------------

pub async fn get_metric_k8s_deployment_cost(
    name: String,
    q: RangeQuery,
) -> Result<Value> {
    let mut dto = build_deployment_cost(Some(name.clone()), q, &[]).await?;

    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    apply_costs(&mut dto, &unit_prices);

    Ok(serde_json::to_value(dto)?)
}

pub async fn get_metric_k8s_deployment_cost_summary(
    name: String,
    q: RangeQuery,
) -> Result<Value> {
    let mut dto = build_deployment_cost(Some(name.clone()), q, &[]).await?;

    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    apply_costs(&mut dto, &unit_prices);

    let summary = build_cost_summary_dto(&dto, MetricScope::Deployment, Some(name), &unit_prices);
    Ok(serde_json::to_value(summary)?)
}

pub async fn get_metric_k8s_deployment_cost_trend(
    name: String,
    q: RangeQuery,
) -> Result<Value> {
    let mut dto = build_deployment_cost(Some(name.clone()), q, &[]).await?;

    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    apply_costs(&mut dto, &unit_prices);

    let trend = build_cost_trend_dto(&dto, MetricScope::Deployment, Some(name))?;
    Ok(serde_json::to_value(trend)?)
}
