use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
};

use crate::api::dto::metrics_dto::RangeQuery;
use crate::core::persistence::info::{
    k8s::pod::{info_pod_entity::InfoPodEntity, info_pod_repository::InfoPodRepository},
    path::info_k8s_pod_dir_path,
};
use crate::core::persistence::info::k8s::pod::info_pod_api_repository_trait::InfoPodApiRepository;
use crate::domain::info::service::info_unit_price_service;

use crate::domain::metric::k8s::common::dto::{
    FilesystemMetricDto, MetricGetResponseDto, MetricScope,
    MetricSeriesDto, NetworkMetricDto, UniversalMetricPointDto,
};
use crate::domain::metric::k8s::common::service_helpers::{
    apply_costs, build_cost_summary_dto, build_cost_trend_dto, build_raw_summary_value,
};

use crate::domain::metric::k8s::pod::service::build_pod_response_from_infos;

// =====================================================================
// HELPERS
// =====================================================================

/// Load pods grouped by namespace from the local repository.
fn load_pods_by_namespace(namespaces: &[String]) -> Result<HashMap<String, Vec<InfoPodEntity>>> {
    let mut map = HashMap::new();
    let dir = info_k8s_pod_dir_path();

    if !dir.exists() {
        return Ok(map);
    }

    let filters: HashSet<String> = namespaces.iter().cloned().collect();
    let allow_all = filters.is_empty();
    let repo = InfoPodRepository::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let pod_uid = entry.file_name().to_string_lossy().to_string();

        if let Ok(pod) = repo.read(&pod_uid) {
            if let Some(ns) = pod.namespace.clone() {
                if allow_all || filters.contains(&ns) {
                    map.entry(ns).or_default().push(pod);
                }
            }
        }
    }

    Ok(map)
}

/// Load all pods for a specific namespace (errors if none found).
fn namespace_pods(ns: &str) -> Result<Vec<InfoPodEntity>> {
    let map = load_pods_by_namespace(&[ns.to_string()])?;

    if let Some(pods) = map.get(ns) {
        if !pods.is_empty() {
            return Ok(pods.clone());
        }
    }

    Err(anyhow!("namespace '{}' has no pods", ns))
}

fn all_pods_for(namespaces: &[String]) -> Result<Vec<InfoPodEntity>> {
    let map = load_pods_by_namespace(namespaces)?;
    Ok(map.into_values().flatten().collect())
}


// =====================================================================
// NAMESPACE AGGREGATION
// =====================================================================

fn build_namespace_response(
    namespace: &str,
    per_pod: &MetricGetResponseDto,
) -> MetricGetResponseDto {
    let all_points: Vec<UniversalMetricPointDto> =
        per_pod.series.iter().flat_map(|s| s.points.clone()).collect();

    let aggregated = aggregate_namespace_points(all_points);

    MetricGetResponseDto {
        start: per_pod.start,
        end: per_pod.end,
        scope: "namespace".to_string(),
        target: Some(namespace.to_string()),
        granularity: per_pod.granularity.clone(),
        series: vec![MetricSeriesDto {
            key: namespace.to_string(),
            name: namespace.to_string(),
            scope: MetricScope::Namespace,
            points: aggregated,
            running_hours: None,
            cost_summary: None,
        }],
        total: None,
        limit: None,
        offset: None,
    }
}


// =====================================================================
// NAMESPACE MULTI-POINT AGGREGATION
// =====================================================================

pub fn aggregate_namespace_points(
    points: Vec<UniversalMetricPointDto>,
) -> Vec<UniversalMetricPointDto> {
    let mut buckets: BTreeMap<DateTime<Utc>, Vec<UniversalMetricPointDto>> = BTreeMap::new();

    for p in points {
        buckets.entry(p.time).or_default().push(p);
    }

    let mut out = Vec::with_capacity(buckets.len());

    for (time, list) in buckets {

        let mut acc = UniversalMetricPointDto {
            time,
            ..Default::default()
        };

        let sum   = |slot: &mut Option<f64>, v: Option<f64>| {
            if let Some(n) = v {
                *slot = Some(slot.unwrap_or(0.0) + n);
            }
        };

        for p in list {
            sum(&mut acc.cpu_memory.cpu_usage_nano_cores, p.cpu_memory.cpu_usage_nano_cores);
            sum(&mut acc.cpu_memory.cpu_usage_core_nano_seconds, p.cpu_memory.cpu_usage_core_nano_seconds);

            sum(&mut acc.cpu_memory.memory_usage_bytes, p.cpu_memory.memory_usage_bytes);
            sum(&mut acc.cpu_memory.memory_working_set_bytes, p.cpu_memory.memory_working_set_bytes);
            sum(&mut acc.cpu_memory.memory_rss_bytes, p.cpu_memory.memory_rss_bytes);
            sum(&mut acc.cpu_memory.memory_page_faults, p.cpu_memory.memory_page_faults);

            if let Some(fs) = p.filesystem.as_ref() {
                let outfs = acc.filesystem.get_or_insert(FilesystemMetricDto::default());
                sum(&mut outfs.used_bytes, fs.used_bytes);
                sum(&mut outfs.capacity_bytes, fs.capacity_bytes);
                sum(&mut outfs.inodes_used, fs.inodes_used);
                sum(&mut outfs.inodes, fs.inodes);
            }

            if let Some(net) = p.network.as_ref() {
                let outnet = acc.network.get_or_insert(NetworkMetricDto::default());
                sum(&mut outnet.rx_bytes, net.rx_bytes);
                sum(&mut outnet.tx_bytes, net.tx_bytes);
                sum(&mut outnet.rx_errors, net.rx_errors);
                sum(&mut outnet.tx_errors, net.tx_errors);
            }
        }

        out.push(acc);
    }

    out
}


// =====================================================================
// RAW METRICS: MULTIPLE NAMESPACES
// =====================================================================

pub async fn get_metric_k8s_namespaces_raw(
    q: RangeQuery,
    namespaces: Vec<String>
) -> Result<Value> {

    let ns_map = load_pods_by_namespace(&namespaces)?;

    let targets =
        if namespaces.is_empty() {
            ns_map.keys().cloned().collect::<Vec<_>>()
        } else {
            namespaces
        };

    let mut series = Vec::new();
    let mut base_resp = None;

    for ns in targets {
        if let Some(pods) = ns_map.get(&ns) {
            if pods.is_empty() {
                continue;
            }
            let per_pod = build_pod_response_from_infos(q.clone(), pods.clone(), Some(ns.clone()))?;
            let aggregated = build_namespace_response(&ns, &per_pod);

            if base_resp.is_none() {
                base_resp = Some(aggregated.clone());
            }
            series.push(aggregated.series[0].clone());
        }
    }

    if let Some(mut base) = base_resp {
        base.series = series;
        base.target = None;

        return Ok(serde_json::to_value(base)?);
    }

    Ok(json!({ "status": "no data" }))
}


// =====================================================================
// RAW METRICS: SINGLE NAMESPACE
// =====================================================================

pub async fn get_metric_k8s_namespace_raw(
    ns: String,
    q: RangeQuery
) -> Result<Value> {

    let pods = namespace_pods(&ns)?;
    let per_pod = build_pod_response_from_infos(q, pods, Some(ns.clone()))?;
    let aggregated = build_namespace_response(&ns, &per_pod);

    Ok(serde_json::to_value(aggregated)?)
}


// =====================================================================
// RAW SUMMARY
// =====================================================================

pub async fn get_metric_k8s_namespaces_raw_summary(
    q: RangeQuery,
    namespaces: Vec<String>
) -> Result<Value> {

    let ns_map = load_pods_by_namespace(&namespaces)?;

    let targets =
        if namespaces.is_empty() {
            ns_map.keys().cloned().collect::<Vec<_>>()
        } else {
            namespaces
        };

    let mut all_pods = Vec::new();

    for ns in targets {
        if let Some(pods) = ns_map.get(&ns) {
            all_pods.extend(pods.clone());
        }
    }

    if all_pods.is_empty() {
        return Ok(json!({ "status": "no data" }));
    }

    let per_pod = build_pod_response_from_infos(q, all_pods.clone(), None)?;
    let aggregated = build_namespace_response("all", &per_pod);

    build_raw_summary_value(&aggregated, MetricScope::Namespace, all_pods.len())
}


pub async fn get_metric_k8s_namespace_raw_summary(
    ns: String,
    q: RangeQuery
) -> Result<Value> {

    let pods = namespace_pods(&ns)?;
    let per_pod = build_pod_response_from_infos(q, pods.clone(), Some(ns.clone()))?;
    let aggregated = build_namespace_response(&ns, &per_pod);

    build_raw_summary_value(&aggregated, MetricScope::Namespace, pods.len())
}



// =====================================================================
// EFFICIENCY (NOT SUPPORTED)
// =====================================================================

pub async fn get_metric_k8s_namespace_raw_efficiency(
    _ns: String, _q: RangeQuery
) -> Result<Value> {
    Ok(json!({
        "status": "not_supported",
        "message": "Namespace efficiency not supported yet"
    }))
}

pub async fn get_metric_k8s_namespaces_raw_efficiency(
    _q: RangeQuery,
    _namespaces: Vec<String>
) -> Result<Value> {
    Ok(json!({
        "status": "not_supported",
        "message": "Namespace efficiency not supported yet"
    }))
}


// =====================================================================
// COST
// =====================================================================

async fn build_namespace_cost(
    namespace: Option<String>,
    q: RangeQuery,
    filter_namespaces: &[String],
) -> Result<MetricGetResponseDto> {

    let pods = match namespace.as_ref() {
        Some(ns) => namespace_pods(ns)?,
        None => all_pods_for(filter_namespaces)?,
    };

    if pods.is_empty() {
        return Err(anyhow!("no pods available for namespace cost calculation"));
    }

    let per_pod = build_pod_response_from_infos(q, pods, namespace.clone())?;

    Ok(build_namespace_response(
        namespace.as_deref().unwrap_or("all"),
        &per_pod,
    ))
}


// MULTIPLE NS
pub async fn get_metric_k8s_namespaces_cost(
    q: RangeQuery,
    namespaces: Vec<String>
) -> Result<Value> {
    let aggregated = build_namespace_cost(None, q, &namespaces).await?;
    Ok(serde_json::to_value(aggregated)?)
}

pub async fn get_metric_k8s_namespace_cost(
    ns: String,
    q: RangeQuery
) -> Result<Value> {
    let aggregated = build_namespace_cost(Some(ns), q, &[]).await?;
    Ok(serde_json::to_value(aggregated)?)
}



// COST SUMMARY

pub async fn get_metric_k8s_namespaces_cost_summary(
    q: RangeQuery,
    namespaces: Vec<String>
) -> Result<Value> {

    let aggregated = build_namespace_cost(None, q.clone(), &namespaces).await?;
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;

    let mut cost_resp = aggregated.clone();
    apply_costs(&mut cost_resp, &unit_prices);

    let dto = build_cost_summary_dto(&cost_resp, MetricScope::Namespace, None, &unit_prices);
    Ok(serde_json::to_value(dto)?)
}

pub async fn get_metric_k8s_namespace_cost_summary(
    ns: String,
    q: RangeQuery
) -> Result<Value> {

    let aggregated = build_namespace_cost(Some(ns.clone()), q.clone(), &[]).await?;
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;

    let mut cost_resp = aggregated.clone();
    apply_costs(&mut cost_resp, &unit_prices);

    let dto = build_cost_summary_dto(
        &cost_resp,
        MetricScope::Namespace,
        Some(ns),
        &unit_prices,
    );

    Ok(serde_json::to_value(dto)?)
}



// COST TREND

pub async fn get_metric_k8s_namespaces_cost_trend(
    q: RangeQuery,
    namespaces: Vec<String>
) -> Result<Value> {

    let aggregated = build_namespace_cost(None, q.clone(), &namespaces).await?;
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;

    let mut cost_resp = aggregated.clone();
    apply_costs(&mut cost_resp, &unit_prices);

    let dto = build_cost_trend_dto(&cost_resp, MetricScope::Namespace, None)?;
    Ok(serde_json::to_value(dto)?)
}

pub async fn get_metric_k8s_namespace_cost_trend(
    ns: String,
    q: RangeQuery
) -> Result<Value> {

    let aggregated = build_namespace_cost(Some(ns.clone()), q.clone(), &[]).await?;
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;

    let mut cost_resp = aggregated.clone();
    apply_costs(&mut cost_resp, &unit_prices);

    let dto =
        build_cost_trend_dto(&cost_resp, MetricScope::Namespace, Some(ns))?;

    Ok(serde_json::to_value(dto)?)
}
