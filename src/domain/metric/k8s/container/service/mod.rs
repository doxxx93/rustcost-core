use anyhow::Result;
use serde_json::Value;
use std::collections::HashSet;

use crate::api::dto::{info_dto::K8sListQuery, metrics_dto::RangeQuery};
use crate::core::persistence::info::fixed::unit_price::info_unit_price_entity::InfoUnitPriceEntity;
use crate::core::persistence::info::k8s::container::info_container_entity::InfoContainerEntity;
use crate::core::persistence::metrics::k8s::container::day::metric_container_day_api_repository_trait::MetricContainerDayApiRepository;
use crate::core::persistence::metrics::k8s::container::hour::metric_container_hour_api_repository_trait::MetricContainerHourApiRepository;
use crate::core::persistence::metrics::k8s::container::metric_container_entity::MetricContainerEntity;
use crate::core::persistence::metrics::k8s::container::minute::metric_container_minute_api_repository_trait::MetricContainerMinuteApiRepository;
use crate::domain::info::service::{info_k8s_container_service, info_unit_price_service};
use crate::domain::metric::k8s::common::dto::{
    CommonMetricValuesDto, FilesystemMetricDto, MetricGetResponseDto, MetricScope, MetricSeriesDto,
    UniversalMetricPointDto,
};
use crate::domain::metric::k8s::common::dto::metric_k8s_raw_summary_dto::MetricRawSummaryResponseDto;
use crate::domain::metric::k8s::common::service_helpers::{
    apply_costs, build_cost_summary_dto, build_cost_trend_dto, build_efficiency_value,
    build_raw_summary_value, resolve_time_window, TimeWindow, BYTES_PER_GB,
};
use crate::domain::metric::k8s::common::util::k8s_metric_repository_resolve::resolve_k8s_metric_repository;
use crate::domain::metric::k8s::common::util::k8s_metric_repository_variant::K8sMetricRepositoryVariant;

fn container_metric_key(info: &InfoContainerEntity) -> Option<String> {
    match (&info.pod_uid, &info.container_name) {
        (Some(pod_uid), Some(container_name)) => Some(format!("{}-{}", pod_uid, container_name)),
        _ => None,
    }
}

fn fetch_container_points(
    repo: &K8sMetricRepositoryVariant,
    container_key: &str,
    window: &TimeWindow,
) -> Result<Vec<UniversalMetricPointDto>> {
    let rows = match repo {
        K8sMetricRepositoryVariant::ContainerMinute(r) => {
            r.get_row_between(window.start, window.end, container_key, None, None)
        }
        K8sMetricRepositoryVariant::ContainerHour(r) => {
            r.get_row_between(window.start, window.end, container_key, None, None)
        }
        K8sMetricRepositoryVariant::ContainerDay(r) => {
            r.get_row_between(window.start, window.end, container_key, None, None)
        }
        _ => Ok(vec![]),
    }?;

    Ok(rows.into_iter().map(metric_container_entity_to_point).collect())
}

fn metric_container_entity_to_point(entity: MetricContainerEntity) -> UniversalMetricPointDto {
    UniversalMetricPointDto {
        time: entity.time,
        cpu_memory: CommonMetricValuesDto {
            cpu_usage_nano_cores: entity.cpu_usage_nano_cores.map(|v| v as f64),
            cpu_usage_core_nano_seconds: entity.cpu_usage_core_nano_seconds.map(|v| v as f64),
            memory_usage_bytes: entity.memory_usage_bytes.map(|v| v as f64),
            memory_working_set_bytes: entity.memory_working_set_bytes.map(|v| v as f64),
            memory_rss_bytes: entity.memory_rss_bytes.map(|v| v as f64),
            memory_page_faults: entity.memory_page_faults.map(|v| v as f64),
        },
        filesystem: Some(FilesystemMetricDto {
            used_bytes: entity.fs_used_bytes.map(|v| v as f64),
            capacity_bytes: entity.fs_capacity_bytes.map(|v| v as f64),
            inodes_used: entity.fs_inodes_used.map(|v| v as f64),
            inodes: entity.fs_inodes.map(|v| v as f64),
        }),
        ..Default::default()
    }
}

async fn build_container_raw_data(
    q: RangeQuery,
    container_keys: Vec<String>,
) -> Result<(MetricGetResponseDto, Vec<InfoContainerEntity>)> {
    let window = resolve_time_window(&q);
    let repo = resolve_k8s_metric_repository(&MetricScope::Container, &window.granularity);

    // 1. Load containers via service (as you already do today)
    let mut container_infos =
        info_k8s_container_service::list_k8s_containers(K8sListQuery {
            namespace: q.namespace.clone(),
            label_selector: None,
            node_name: None,
        })
            .await?;

    // 1-1. If a list of container_keys is provided, filter by them
    if !container_keys.is_empty() {
        let wanted: HashSet<String> = container_keys.into_iter().collect();
        container_infos.retain(|c| {
            container_metric_key(c)
                .map(|k| wanted.contains(&k))
                .unwrap_or(false)
        });
    }

    // 2. Apply filtering: team, service, env
    let matches = |value: &Option<String>, filter: &str| {
        value
            .as_deref()
            .map(|v| {
                v.split(',')
                    .any(|x| x.trim().eq_ignore_ascii_case(filter.trim()))
            })
            .unwrap_or(false)
    };

    if let Some(ref team) = q.team {
        container_infos.retain(|c| matches(&c.team, team));
    }
    if let Some(ref service) = q.service {
        container_infos.retain(|c| matches(&c.service, service));
    }
    if let Some(ref env) = q.env {
        container_infos.retain(|c| matches(&c.env, env));
    }

    // 3. Build metric series
    let mut series = Vec::new();
    for container in container_infos.iter() {
        if let Some(key) = container_metric_key(container) {
            let points = fetch_container_points(&repo, &key, &window)?;
            let name = container
                .container_name
                .clone()
                .unwrap_or_else(|| key.clone());

            series.push(MetricSeriesDto {
                key,
                name,
                scope: MetricScope::Container,
                points,
                running_hours: None,
                cost_summary: None,
            });
        }
    }

    let response = MetricGetResponseDto {
        start: window.start,
        end: window.end,
        scope: "container".to_string(),
        target: None, // target only used for "single" calls; we keep it None here
        granularity: window.granularity.clone(),
        series,
        total: None,
        limit: None,
        offset: None,
    };

    Ok((response, container_infos))
}

fn sum_container_requests(containers: &[InfoContainerEntity]) -> (f64, f64) {
    let mut total_cpu = 0.0;
    let mut total_mem_gb = 0.0;

    for container in containers {
        total_cpu += container.cpu_request_millicores.unwrap_or(0) as f64 / 1000.0;
        total_mem_gb += container.memory_request_bytes.unwrap_or(0) as f64 / BYTES_PER_GB;
    }

    (total_cpu, total_mem_gb)
}

async fn build_container_cost_response(
    q: RangeQuery,
    container_keys: Vec<String>,
    unit_prices: InfoUnitPriceEntity,
) -> Result<MetricGetResponseDto> {
    let (mut response, _) = build_container_raw_data(q, container_keys).await?;
    apply_costs(&mut response, &unit_prices);
    Ok(response)
}

// ======================================================================
// PUBLIC APIS (MATCH DELEGATE SIGNATURES)
// ======================================================================

// ---------- RAW: multiple containers ----------

pub async fn get_metric_k8s_containers_raw(
    q: RangeQuery,
    container_keys: Vec<String>,
) -> Result<Value> {
    let (response, _) = build_container_raw_data(q, container_keys).await?;
    Ok(serde_json::to_value(response)?)
}

pub async fn get_metric_k8s_containers_raw_summary(
    q: RangeQuery,
    container_keys: Vec<String>,
) -> Result<Value> {
    let (response, containers) = build_container_raw_data(q, container_keys).await?;
    build_raw_summary_value(&response, MetricScope::Container, containers.len())
}

pub async fn get_metric_k8s_containers_raw_efficiency(
    q: RangeQuery,
    container_keys: Vec<String>,
) -> Result<Value> {
    let (response, containers) = build_container_raw_data(q.clone(), container_keys).await?;
    let summary_value =
        build_raw_summary_value(&response, MetricScope::Container, containers.len())?;
    let summary: MetricRawSummaryResponseDto = serde_json::from_value(summary_value)?;

    let (total_cpu, total_mem_gb) = sum_container_requests(&containers);
    let total_storage_gb = summary.summary.max_storage_gb;

    build_efficiency_value(
        summary,
        MetricScope::Container,
        total_cpu,
        total_mem_gb,
        total_storage_gb,
    )
}

// ---------- RAW: single container (id) ----------

pub async fn get_metric_k8s_container_raw(
    id: String,
    q: RangeQuery,
) -> Result<Value> {
    let keys = vec![id];
    let (response, _) = build_container_raw_data(q, keys).await?;
    Ok(serde_json::to_value(response)?)
}

pub async fn get_metric_k8s_container_raw_summary(
    id: String,
    q: RangeQuery,
) -> Result<Value> {
    let keys = vec![id];
    let (response, _) = build_container_raw_data(q, keys).await?;
    build_raw_summary_value(&response, MetricScope::Container, 1)
}

pub async fn get_metric_k8s_container_raw_efficiency(
    id: String,
    q: RangeQuery,
) -> Result<Value> {
    let keys = vec![id];
    let (response, containers) = build_container_raw_data(q.clone(), keys).await?;
    let summary_value = build_raw_summary_value(&response, MetricScope::Container, 1)?;
    let summary: MetricRawSummaryResponseDto = serde_json::from_value(summary_value)?;

    let (total_cpu, total_mem_gb) = sum_container_requests(&containers);
    let total_storage_gb = summary.summary.max_storage_gb;

    build_efficiency_value(
        summary,
        MetricScope::Container,
        total_cpu,
        total_mem_gb,
        total_storage_gb,
    )
}

// ---------- COST: multiple containers ----------

pub async fn get_metric_k8s_containers_cost(
    q: RangeQuery,
    container_keys: Vec<String>,
) -> Result<Value> {
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    let response = build_container_cost_response(q, container_keys, unit_prices).await?;
    Ok(serde_json::to_value(response)?)
}

pub async fn get_metric_k8s_containers_cost_summary(
    q: RangeQuery,
    container_keys: Vec<String>,
) -> Result<Value> {
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    let response =
        build_container_cost_response(q, container_keys, unit_prices.clone()).await?;
    let dto =
        build_cost_summary_dto(&response, MetricScope::Container, None, &unit_prices);
    Ok(serde_json::to_value(dto)?)
}

pub async fn get_metric_k8s_containers_cost_trend(
    q: RangeQuery,
    container_keys: Vec<String>,
) -> Result<Value> {
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    let response = build_container_cost_response(q, container_keys, unit_prices).await?;
    let dto = build_cost_trend_dto(&response, MetricScope::Container, None)?;
    Ok(serde_json::to_value(dto)?)
}

// ---------- COST: single container (id) ----------

pub async fn get_metric_k8s_container_cost(
    id: String,
    q: RangeQuery,
) -> Result<Value> {
    let keys = vec![id.clone()];
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    let response = build_container_cost_response(q, keys, unit_prices).await?;
    Ok(serde_json::to_value(response)?)
}

pub async fn get_metric_k8s_container_cost_summary(
    id: String,
    q: RangeQuery,
) -> Result<Value> {
    let keys = vec![id.clone()];
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    let response =
        build_container_cost_response(q, keys, unit_prices.clone()).await?;
    let dto =
        build_cost_summary_dto(&response, MetricScope::Container, Some(id), &unit_prices);
    Ok(serde_json::to_value(dto)?)
}

pub async fn get_metric_k8s_container_cost_trend(
    id: String,
    q: RangeQuery,
) -> Result<Value> {
    let keys = vec![id.clone()];
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    let response = build_container_cost_response(q, keys, unit_prices).await?;
    let dto = build_cost_trend_dto(&response, MetricScope::Container, Some(id))?;
    Ok(serde_json::to_value(dto)?)
}
