use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::api::dto::metrics_dto::RangeQuery;
use crate::core::persistence::info::fixed::unit_price::info_unit_price_entity::InfoUnitPriceEntity;
use crate::core::persistence::info::k8s::node::info_node_api_repository_trait::InfoNodeApiRepository;
use crate::core::persistence::info::k8s::node::info_node_entity::InfoNodeEntity;
use crate::core::persistence::info::k8s::node::info_node_repository::InfoNodeRepository;
use crate::core::persistence::metrics::k8s::node::day::metric_node_day_repository::MetricNodeDayRepository;
use crate::core::persistence::metrics::k8s::node::hour::metric_node_hour_api_repository_trait::MetricNodeHourApiRepository;
use crate::core::persistence::metrics::k8s::node::hour::metric_node_hour_repository::MetricNodeHourRepository;
use crate::core::persistence::metrics::k8s::node::metric_node_entity::MetricNodeEntity;
use crate::core::persistence::metrics::k8s::node::minute::metric_node_minute_api_repository_trait::MetricNodeMinuteApiRepository;
use crate::domain::common::service::day_granularity::split_day_granularity_rows;
use crate::domain::info::service::{info_unit_price_service};
use crate::domain::metric::k8s::common::dto::{CommonMetricValuesDto, FilesystemMetricDto, MetricGetResponseDto, MetricGranularity, MetricScope, MetricSeriesDto, NetworkMetricDto, UniversalMetricPointDto};
use crate::domain::metric::k8s::common::dto::metric_k8s_raw_summary_dto::MetricRawSummaryResponseDto;
use crate::domain::metric::k8s::common::service_helpers::{apply_costs, apply_node_costs, build_cost_summary_dto, build_cost_trend_dto, build_efficiency_value, build_node_cost_summary_dto, build_raw_summary_value, resolve_time_window, TimeWindow, BYTES_PER_GB};
use crate::domain::metric::k8s::common::util::k8s_metric_repository_resolve::resolve_k8s_metric_repository;
use crate::domain::metric::k8s::common::util::k8s_metric_repository_variant::K8sMetricRepositoryVariant;

fn fetch_node_points(
    repo: &K8sMetricRepositoryVariant,
    node_name: &str,
    window: &TimeWindow,
) -> Result<(Vec<UniversalMetricPointDto>, f64)> {

    match repo {
        // --------------------
        // Minute
        // --------------------
        K8sMetricRepositoryVariant::NodeMinute(r) => {
            let rows = r.get_row_between(node_name, window.start, window.end)?;
            let running_hours = rows.len() as f64 / 60.0;

            let points = rows
                .into_iter()
                .map(metric_node_entity_to_point)
                .collect();

            Ok((points, running_hours))
        }

        // --------------------
        // Hour
        // --------------------
        K8sMetricRepositoryVariant::NodeHour(r) => {
            let rows = r.get_row_between(node_name, window.start, window.end)?;
            let running_hours = rows.len() as f64;

            let points = rows
                .into_iter()
                .map(metric_node_entity_to_point)
                .collect();

            Ok((points, running_hours))
        }

        // --------------------
        // Day
        // --------------------
        K8sMetricRepositoryVariant::NodeDay(_) => {
            let day_repo = MetricNodeDayRepository::new();
            let hour_repo = MetricNodeHourRepository::new();

            let split = split_day_granularity_rows(
                node_name,
                window,
                &day_repo,
                &hour_repo,
            )?;

            let running_hours =
                split.start_hour_rows.len() as f64 +
                    split.end_hour_rows.len() as f64 +
                    split.middle_day_rows.len() as f64 * 24.0;

            let mut rows = Vec::new();
            rows.extend(split.start_hour_rows);
            rows.extend(split.middle_day_rows);
            rows.extend(split.end_hour_rows);

            let points = rows
                .into_iter()
                .map(metric_node_entity_to_point)
                .collect();

            Ok((points, running_hours))
        }

        _ => Ok((vec![], 0.0)),
    }
}

fn metric_node_entity_to_point(entity: MetricNodeEntity) -> UniversalMetricPointDto {
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
        network: Some(NetworkMetricDto {
            rx_bytes: entity.network_physical_rx_bytes.map(|v| v as f64),
            tx_bytes: entity.network_physical_tx_bytes.map(|v| v as f64),
            rx_errors: entity.network_physical_rx_errors.map(|v| v as f64),
            tx_errors: entity.network_physical_tx_errors.map(|v| v as f64),
        }),
        ..Default::default()
    }
}

async fn build_node_raw_data(
    q: RangeQuery,
    node_names: Vec<String>,
) -> Result<(MetricGetResponseDto, Vec<InfoNodeEntity>)> {

    // 1️⃣ Resolve metric window + repository
    let window = resolve_time_window(&q);
    let metric_repo = resolve_k8s_metric_repository(&MetricScope::Node, &window.granularity);

    // 2️⃣ Load node metadata from repo (POD MODEL)
    let info_repo = InfoNodeRepository::new();
    let mut node_infos = Vec::new();

    for name in node_names {
        if let Ok(info) = info_repo.read(&name) {
            node_infos.push(info);
        }
    }

    // 3️⃣ Apply filters
    let matches = |value: &Option<String>, filter: &str| {
        value.as_deref()
            .map(|v| v.split(',').any(|x| x.trim().eq_ignore_ascii_case(filter)))
            .unwrap_or(false)
    };

    if let Some(team) = &q.team {
        node_infos.retain(|n| matches(&n.team, team));
    }
    if let Some(service) = &q.service {
        node_infos.retain(|n| matches(&n.service, service));
    }
    if let Some(env) = &q.env {
        node_infos.retain(|n| matches(&n.env, env));
    }

    // 4️⃣ Sorting
    match q.sort.as_deref() {
        Some("cpu") => node_infos.sort_by(|a, b| a.cpu_capacity_cores.cmp(&b.cpu_capacity_cores)),
        Some("memory") => node_infos.sort_by(|a, b| a.memory_capacity_bytes.cmp(&b.memory_capacity_bytes)),
        Some("ready") => node_infos.sort_by(|a, b| a.ready.cmp(&b.ready)),
        Some("ip") => node_infos.sort_by(|a, b| a.internal_ip.cmp(&b.internal_ip)),
        _ => node_infos.sort_by(|a, b| a.node_name.cmp(&b.node_name)),
    }

    // 5️⃣ Pagination
    let offset = q.offset.unwrap_or(0);
    let limit = q.limit.unwrap_or(100);
    let total = node_infos.len();

    let page_slice = node_infos
        .iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();

    // 6️⃣ Build metric series (from correct metric repo)
    let mut series = Vec::new();
    for node in &page_slice {
        let name = node
            .node_name
            .clone()
            .ok_or_else(|| anyhow!("Node record missing name"))?;

        let (points, running_hours) = fetch_node_points(&metric_repo, &name, &window)?;
        series.push(MetricSeriesDto {
            key: name.clone(),
            name: name.clone(),
            scope: MetricScope::Node,
            points,
            running_hours: Some(running_hours),
            cost_summary: None,
        });
    }

    // 7️⃣ Build response
    let response = MetricGetResponseDto {
        start: window.start,
        end: window.end,
        scope: "node".to_string(),
        target: None,
        granularity: window.granularity,
        series,
        total: Some(total),
        limit: Some(limit),
        offset: Some(offset),
    };

    Ok((response, page_slice))
}

fn sum_node_allocations(nodes: &[InfoNodeEntity]) -> (f64, f64, f64) {
    let mut total_cpu = 0.0;
    let mut total_mem_bytes = 0.0;
    let mut total_storage_bytes = 0.0;

    for node in nodes {
        total_cpu += node.cpu_allocatable_cores.unwrap_or(0) as f64;
        total_mem_bytes += node.memory_allocatable_bytes.unwrap_or(0) as f64;
        total_storage_bytes += node.ephemeral_storage_allocatable_bytes.unwrap_or(0) as f64;
    }

    (
        total_cpu,
        total_mem_bytes / BYTES_PER_GB,
        total_storage_bytes / BYTES_PER_GB,
    )
}


pub async fn get_metric_k8s_nodes_raw(q: RangeQuery, node_names: Vec<String>) -> Result<Value> {
    let (response, _) = build_node_raw_data(q, node_names).await?;
    Ok(serde_json::to_value(response)?)
}

pub async fn get_metric_k8s_nodes_raw_summary(q: RangeQuery, node_names: Vec<String>) -> Result<Value> {
    let (response, node_infos) = build_node_raw_data(q, node_names).await?;
    build_raw_summary_value(&response, MetricScope::Node, node_infos.len())
}

pub async fn get_metric_k8s_nodes_raw_efficiency(q: RangeQuery, node_names: Vec<String>) -> Result<Value> {
    let (summary_value, node_infos) = {
        let (response, infos) = build_node_raw_data(q.clone(), node_names).await?;
        let summary_json = build_raw_summary_value(&response, MetricScope::Node, infos.len())?;
        (summary_json, infos)
    };

    let summary: MetricRawSummaryResponseDto = serde_json::from_value(summary_value)?;
    let (total_cpu, total_mem, total_storage) = sum_node_allocations(&node_infos);
    build_efficiency_value(summary, MetricScope::Node, total_cpu, total_mem, total_storage)
}

pub async fn get_metric_k8s_node_raw(node_name: String, q: RangeQuery) -> Result<Value> {
    let names = vec![node_name];
    let (response, _) = build_node_raw_data(q, names).await?;
    Ok(serde_json::to_value(response)?)
}

pub async fn get_metric_k8s_node_raw_summary(node_name: String, q: RangeQuery) -> Result<Value> {
    let names = vec![node_name];
    let (response, _) = build_node_raw_data(q, names).await?;
    build_raw_summary_value(&response, MetricScope::Node, 1)
}

pub async fn get_metric_k8s_node_raw_efficiency(node_name: String, q: RangeQuery) -> Result<Value> {
    let names = vec![node_name];
    let (response, node_infos) = build_node_raw_data(q.clone(), names).await?;
    let summary_value = build_raw_summary_value(&response, MetricScope::Node, 1)?;
    let summary: MetricRawSummaryResponseDto = serde_json::from_value(summary_value)?;
    let (total_cpu, total_mem, total_storage) = sum_node_allocations(&node_infos);
    build_efficiency_value(summary, MetricScope::Node, total_cpu, total_mem, total_storage)
}

async fn build_node_cost_response(
    q: RangeQuery,
    node_names: Vec<String>,
    unit_prices: InfoUnitPriceEntity,
) -> Result<MetricGetResponseDto> {
    let (mut response, node_infos) = build_node_raw_data(q, node_names).await?;
    apply_node_costs(&mut response, &unit_prices, &node_infos);

    Ok(response)
}

async fn build_node_cost_response_v2(
    q: RangeQuery,
    node_names: Vec<String>,
    unit_prices: InfoUnitPriceEntity,
) -> Result<MetricGetResponseDto> {
    let (mut response, node_infos) = build_node_raw_data(q, node_names).await?;
    apply_node_costs(&mut response, &unit_prices, &node_infos);

    Ok(response)
}

pub async fn get_metric_k8s_nodes_cost(q: RangeQuery, node_names: Vec<String>) -> Result<Value> {
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    let response = build_node_cost_response(q, node_names, unit_prices).await?;
    Ok(serde_json::to_value(response)?)
}

pub async fn get_metric_k8s_nodes_cost_summary(q: RangeQuery, node_names: Vec<String>) -> Result<Value> {
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    let response = build_node_cost_response(q, node_names, unit_prices.clone()).await?;
    let dto = build_node_cost_summary_dto(&response, MetricScope::Node, None, &unit_prices);
    Ok(serde_json::to_value(dto)?)
}

pub async fn get_metric_k8s_nodes_cost_summary_v2(q: RangeQuery, node_names: Vec<String>) -> Result<Value> {
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    let response = build_node_cost_response(q, node_names, unit_prices.clone()).await?;
    let dto = build_cost_summary_dto(&response, MetricScope::Node, None, &unit_prices);
    Ok(serde_json::to_value(dto)?)
}

pub async fn get_metric_k8s_nodes_cost_trend(q: RangeQuery, node_names: Vec<String>) -> Result<Value> {
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    let response = build_node_cost_response(q, node_names, unit_prices).await?;
    let dto = build_cost_trend_dto(&response, MetricScope::Node, None)?;
    Ok(serde_json::to_value(dto)?)
}

pub async fn get_metric_k8s_node_cost(node_name: String, q: RangeQuery) -> Result<Value> {
    let names = vec![node_name];
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    let response = build_node_cost_response(q, names, unit_prices).await?;
    Ok(serde_json::to_value(response)?)
}

pub async fn get_metric_k8s_node_cost_summary(node_name: String, q: RangeQuery) -> Result<Value> {
    let names = vec![node_name.clone()];
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    let response = build_node_cost_response(q, names, unit_prices.clone()).await?;
    let dto = build_cost_summary_dto(&response, MetricScope::Node, Some(node_name), &unit_prices);
    Ok(serde_json::to_value(dto)?)
}

pub async fn get_metric_k8s_node_cost_trend(node_name: String, q: RangeQuery) -> Result<Value> {
    let names = vec![node_name.clone()];
    let unit_prices = info_unit_price_service::get_info_unit_prices().await?;
    let response = build_node_cost_response(q, names, unit_prices).await?;
    let dto = build_cost_trend_dto(&response, MetricScope::Node, Some(node_name))?;
    Ok(serde_json::to_value(dto)?)
}
