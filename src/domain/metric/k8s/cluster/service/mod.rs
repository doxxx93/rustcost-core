use crate::api::dto::metrics_dto::RangeQuery;
use crate::core::persistence::info::fixed::unit_price::info_unit_price_entity::InfoUnitPriceEntity;
use crate::core::persistence::info::k8s::node::info_node_api_repository_trait::InfoNodeApiRepository;
use crate::core::persistence::info::k8s::node::info_node_entity::InfoNodeEntity;
use crate::core::persistence::metrics::k8s::node::day::metric_node_day_api_repository_trait::MetricNodeDayApiRepository;
use crate::core::persistence::metrics::k8s::node::day::metric_node_day_repository::MetricNodeDayRepository;
use crate::core::persistence::metrics::k8s::node::hour::metric_node_hour_api_repository_trait::MetricNodeHourApiRepository;
use crate::core::persistence::metrics::k8s::node::hour::metric_node_hour_repository::MetricNodeHourRepository;
use crate::core::persistence::metrics::k8s::node::minute::metric_node_minute_api_repository_trait::MetricNodeMinuteApiRepository;
use crate::domain::metric::k8s::common::dto::metric_k8s_raw_efficiency_dto::{MetricRawEfficiencyDto, MetricRawEfficiencyResponseDto};
use crate::domain::metric::k8s::common::dto::metric_k8s_raw_summary_dto::{MetricRawSummaryDto, MetricRawSummaryResponseDto};
use crate::domain::metric::k8s::common::dto::{CommonMetricValuesDto, FilesystemMetricDto, MetricGetResponseDto, MetricGranularity, MetricScope, MetricSeriesDto, NetworkMetricDto, UniversalMetricPointDto};
use crate::domain::metric::k8s::common::service_helpers::{apply_costs, build_cost_summary_dto, build_cost_trend_dto, resolve_time_window};
use crate::domain::common::service::day_granularity::{split_day_granularity_rows};
use crate::domain::metric::k8s::common::util::k8s_metric_repository_resolve::resolve_k8s_metric_repository;
use crate::domain::metric::k8s::common::util::k8s_metric_repository_variant::K8sMetricRepositoryVariant;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use crate::domain::metric::k8s::common::dto::metric_k8s_cost_summary_dto::{MetricCostSummaryDto, MetricCostSummaryResponseDto};

pub async fn get_metric_k8s_cluster_cost_summary(
    node_names: Vec<String>,
    unit_prices: InfoUnitPriceEntity,
    q: RangeQuery,
) -> Result<Value> {
    let window = resolve_time_window(&q);
    
    let mut total_cpu_cost = 0.0;
    let mut total_memory_cost = 0.0;
    let mut total_storage_cost = 0.0;

    let info_repo =
        crate::core::persistence::info::k8s::node::info_node_repository::InfoNodeRepository::new();

    let metric_repo =
        resolve_k8s_metric_repository(&MetricScope::Node, &window.granularity);

    for node_name in node_names {
        let running_hours = match window.granularity {

            MetricGranularity::Minute => {
                let rows = match &metric_repo {
                    K8sMetricRepositoryVariant::NodeMinute(r) =>
                        r.get_row_between(&node_name, window.start, window.end),
                    _ => Ok(vec![]),
                }?;
                rows.len() as f64 / 60.0
            }

            MetricGranularity::Hour => {
                let rows = match &metric_repo {
                    K8sMetricRepositoryVariant::NodeHour(r) =>
                        MetricNodeHourApiRepository::get_row_between(
                            r,
                            &node_name,
                            window.start,
                            window.end,
                        ),
                    _ => Ok(vec![]),
                }?;
                rows.len() as f64
            }

            MetricGranularity::Day => {
                let day_repo = MetricNodeDayRepository::new();
                let hour_repo = MetricNodeHourRepository::new();

                let split_row = split_day_granularity_rows(
                    &node_name,
                    &window,
                    &day_repo,
                    &hour_repo,
                )?;

                split_row.start_hour_rows.len() as f64 + split_row.end_hour_rows.len() as f64 + split_row.middle_day_rows.len() as f64 * 24.0
            }
        };

        if running_hours <= 0.0 {
            continue;
        }

        let node_info = match info_repo.read(&node_name) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let cpu_cores = node_info.cpu_capacity_cores.unwrap_or(0) as f64;
        let memory_gb = node_info.memory_capacity_bytes.unwrap_or(0) as f64 / 1_073_741_824.0;
        let storage_gb = node_info.ephemeral_storage_capacity_bytes.unwrap_or(0) as f64 / 1_073_741_824.0;

        total_cpu_cost += cpu_cores * running_hours * unit_prices.cpu_core_hour;
        total_memory_cost += memory_gb * running_hours * unit_prices.memory_gb_hour;
        total_storage_cost += storage_gb * running_hours * unit_prices.storage_gb_hour;
    }

    let summary = MetricCostSummaryDto {
        cpu_cost_usd: total_cpu_cost,
        memory_cost_usd: total_memory_cost,
        ephemeral_storage_cost_usd: total_storage_cost,
        persistent_storage_cost_usd: 0.0,
        total_cost_usd: total_cpu_cost + total_memory_cost + total_storage_cost,
        network_cost_usd: 0.0,
    };

    let resp = MetricCostSummaryResponseDto {
        start: window.start,
        end: window.end,
        scope: MetricScope::Cluster,
        target: None,
        granularity: window.granularity.clone(),
        summary,
    };

    Ok(serde_json::to_value(resp)?)
}

pub async fn get_metric_k8s_cluster_raw(
    node_names: Vec<String>,
    q: RangeQuery,
) -> Result<Value, anyhow::Error> {

    let window = resolve_time_window(&q);
    let repo = resolve_k8s_metric_repository(&MetricScope::Node, &window.granularity);

    let mut aggregated_points: Vec<UniversalMetricPointDto> = Vec::new();

    for node_name in &node_names {

        // Load per-node metric rows
        let rows = match &repo {
            K8sMetricRepositoryVariant::NodeMinute(r) => {
                r.get_row_between(node_name, window.start, window.end)
            }
            K8sMetricRepositoryVariant::NodeHour(r) => {
                MetricNodeHourApiRepository::get_row_between(
                    r,
                    &node_name,
                    window.start,
                    window.end,
                )
            }
            K8sMetricRepositoryVariant::NodeDay(r) => {
                MetricNodeDayApiRepository::get_row_between(
                    r,
                    &node_name,
                    window.start,
                    window.end,
                )
            }
            K8sMetricRepositoryVariant::PodMinute(_)
            | K8sMetricRepositoryVariant::PodHour(_)
            | K8sMetricRepositoryVariant::PodDay(_)
            | K8sMetricRepositoryVariant::ContainerMinute(_)
            | K8sMetricRepositoryVariant::ContainerHour(_)
            | K8sMetricRepositoryVariant::ContainerDay(_) => Err(anyhow!(
                "Cluster node metrics require a node repository for granularity {:?}",
                window.granularity
            )),
        }
        .unwrap_or_else(|err| {
            tracing::warn!("Failed loading metrics for {}: {}", node_name, err);
            vec![]
        });

        // Convert to universal struct ??preserve missing values (None/null)
        aggregated_points.extend(rows.into_iter().map(|m| {
            UniversalMetricPointDto {
                time: m.time,
                cpu_memory: CommonMetricValuesDto {
                    cpu_usage_nano_cores: m.cpu_usage_nano_cores.map(|v| v as f64),
                    cpu_usage_core_nano_seconds: m.cpu_usage_core_nano_seconds.map(|v| v as f64),
                    memory_usage_bytes: m.memory_usage_bytes.map(|v| v as f64),
                    memory_working_set_bytes: m.memory_working_set_bytes.map(|v| v as f64),
                    memory_rss_bytes: m.memory_rss_bytes.map(|v| v as f64),
                    memory_page_faults: m.memory_page_faults.map(|v| v as f64),
                },
                filesystem: Some(FilesystemMetricDto {
                    used_bytes: m.fs_used_bytes.map(|v| v as f64),
                    capacity_bytes: m.fs_capacity_bytes.map(|v| v as f64),
                    inodes_used: m.fs_inodes_used.map(|v| v as f64),
                    inodes: m.fs_inodes.map(|v| v as f64),
                }),
                network: Some(NetworkMetricDto {
                    rx_bytes: m.network_physical_rx_bytes.map(|v| v as f64),
                    tx_bytes: m.network_physical_tx_bytes.map(|v| v as f64),
                    rx_errors: m.network_physical_rx_errors.map(|v| v as f64),
                    tx_errors: m.network_physical_tx_errors.map(|v| v as f64),
                }),
                storage: None,
                cost: None,
            }
        }));
    }

    // Aggregate multiple nodes ??cluster values
    let cluster_points = aggregate_cluster_points(aggregated_points);

    let response = MetricGetResponseDto {
        start: window.start,
        end: window.end,
        scope: "cluster".into(),
        target: None,
        granularity: window.granularity,
        series: vec![MetricSeriesDto {
            key: "cluster".into(),
            name: "cluster".into(),
            scope: MetricScope::Cluster,
            points: cluster_points,
            running_hours: None,
            cost_summary: None,
        }],
        // Cluster API does not paginate output
        total: None,
        limit: None,
        offset: None,
    };

    Ok(serde_json::to_value(response)?)
}


/// Summarize raw cluster resource usage (CPU, memory, storage, network)
// use tracing::{debug, warn}; // Uncomment if you're using `tracing`
/// Summarize raw cluster resource usage (CPU, memory, storage, network).
///
/// - Fetches raw metrics for the given nodes and time range
/// - Computes averages and max values across all valid samples
/// - Handles missing data gracefully (skips missing/NaN/negative samples)
/// - For network, treats rx/tx as cumulative counters and aggregates deltas
pub async fn get_metric_k8s_cluster_raw_summary(
    node_names: Vec<String>,
    q: RangeQuery,
) -> Result<Value> {
    const NANOCORES_PER_CORE: f64 = 1_000_000_000.0;
    const BYTES_PER_GIB: f64 = 1_073_741_824.0;

    // 1️⃣ Retrieve the raw metrics for the time range
    let raw_value = get_metric_k8s_cluster_raw(node_names.clone(), q).await?;
    let cluster_metrics: MetricGetResponseDto = serde_json::from_value(raw_value)?;

    if cluster_metrics.series.is_empty() {
        return Ok(json!({ "status": "no data" }));
    }

    // 2️⃣ Prepare accumulators (per-metric sample counts)
    let mut total_cpu_cores = 0.0;
    let mut max_cpu_cores = 0.0;
    let mut cpu_samples = 0u64;

    let mut total_mem_gib = 0.0;
    let mut max_mem_gib = 0.0;
    let mut mem_samples = 0u64;

    let mut total_storage_gib = 0.0;
    let mut max_storage_gib = 0.0;
    let mut storage_samples = 0u64;

    // For network, we treat rx/tx as cumulative counters and accumulate deltas.
    let mut total_network_bytes = 0.0;
    let mut max_network_gib_per_interval = 0.0;
    let mut network_intervals = 0u64;

    let mut has_any_point = false;

    // 3️⃣ Aggregate usage across all metric points
    for series in &cluster_metrics.series {
        // For network deltas within this series
        let mut prev_net_bytes: Option<f64> = None;

        for point in &series.points {
            has_any_point = true;

            // --- CPU ---
            if let Some(nano_cores) = point.cpu_memory.cpu_usage_nano_cores {
                let cores = nano_cores / NANOCORES_PER_CORE;

                if cores.is_finite() && cores >= 0.0 {
                    total_cpu_cores += cores;
                    cpu_samples += 1;

                    if cores > max_cpu_cores {
                        max_cpu_cores = cores;
                    }
                } else {
                    // warn!("Invalid CPU value: {}", cores);
                }
            } else {
                // debug!("Missing cpu_usage_nano_cores for a point");
            }

            // --- Memory ---
            if let Some(mem_bytes) = point.cpu_memory.memory_usage_bytes {
                let mem_gib = mem_bytes / BYTES_PER_GIB;

                if mem_gib.is_finite() && mem_gib >= 0.0 {
                    total_mem_gib += mem_gib;
                    mem_samples += 1;

                    if mem_gib > max_mem_gib {
                        max_mem_gib = mem_gib;
                    }
                } else {
                    // warn!("Invalid memory value: {}", mem_gib);
                }
            } else {
                // debug!("Missing memory_usage_bytes for a point");
            }

            // --- Storage ---
            if let Some(fs) = point.filesystem.as_ref() {
                if let Some(used_bytes) = fs.used_bytes {
                    let fs_gib = used_bytes / BYTES_PER_GIB;

                    if fs_gib.is_finite() && fs_gib >= 0.0 {
                        total_storage_gib += fs_gib;
                        storage_samples += 1;

                        if fs_gib > max_storage_gib {
                            max_storage_gib = fs_gib;
                        }
                    } else {
                        // warn!("Invalid filesystem.used_bytes value: {}", fs_gib);
                    }
                } else {
                    // debug!("Missing filesystem.used_bytes for a point");
                }
            }

            // --- Network (counters -> deltas) ---
            if let Some(net) = point.network.as_ref() {
                let rx = net.rx_bytes.unwrap_or(0.0);
                let tx = net.tx_bytes.unwrap_or(0.0);
                let combined = rx + tx;

                if combined.is_finite() && combined >= 0.0 {
                    if let Some(prev) = prev_net_bytes {
                        if combined >= prev {
                            let delta_bytes = combined - prev;
                            total_network_bytes += delta_bytes;
                            network_intervals += 1;

                            let delta_gib = delta_bytes / BYTES_PER_GIB;
                            if delta_gib > max_network_gib_per_interval {
                                max_network_gib_per_interval = delta_gib;
                            }
                        } else {
                            // Counter reset or rollover
                            // warn!("Network counters decreased (possible reset)");
                        }
                    }

                    prev_net_bytes = Some(combined);
                } else {
                    // warn!("Invalid network counter value: {}", combined);
                }
            }
        }
    }

    if !has_any_point {
        return Ok(json!({ "status": "no data" }));
    }

    // 4️⃣ Compute averages (defensive against zero samples)
    let avg_cpu_cores = if cpu_samples > 0 {
        total_cpu_cores / cpu_samples as f64
    } else {
        // warn!("No CPU samples found while summarizing cluster metrics");
        0.0
    };

    let avg_memory_gb = if mem_samples > 0 {
        total_mem_gib / mem_samples as f64
    } else {
        // warn!("No memory samples found while summarizing cluster metrics");
        0.0
    };

    let avg_storage_gb = if storage_samples > 0 {
        total_storage_gib / storage_samples as f64
    } else {
        // warn!("No storage samples found while summarizing cluster metrics");
        0.0
    };

    // For network, we averaged delta per interval (still in GiB)
    let avg_network_gb = if network_intervals > 0 {
        (total_network_bytes / BYTES_PER_GIB) / network_intervals as f64
    } else {
        // warn!("No network intervals found while summarizing cluster metrics");
        0.0
    };

    let max_network_gb = max_network_gib_per_interval;

    // 5️⃣ Build summary DTO
    let summary = MetricRawSummaryDto {
        avg_cpu_cores,
        max_cpu_cores,
        avg_memory_gb,
        max_memory_gb: max_mem_gib,
        avg_storage_gb,
        max_storage_gb: max_storage_gib,
        avg_network_gb,
        max_network_gb,
        node_count: node_names.len(),
    };

    let response = MetricRawSummaryResponseDto {
        start: cluster_metrics.start,
        end: cluster_metrics.end,
        scope: MetricScope::Cluster,
        granularity: cluster_metrics.granularity,
        summary,
    };

    Ok(serde_json::to_value(response)?)
}


/// Compute derived cluster costs based on node metrics and unit prices
pub async fn get_metric_k8s_cluster_cost(
    node_names: Vec<String>,
    unit_prices: InfoUnitPriceEntity,
    q: RangeQuery,
) -> Result<Value> {
    // Get raw cluster metrics first
    let raw_value = get_metric_k8s_cluster_raw(node_names, q).await?;
    let mut resp: MetricGetResponseDto = serde_json::from_value(raw_value)?;

    apply_costs(&mut resp, &unit_prices);

    Ok(serde_json::to_value(resp)?)
}

/// Analyze cluster cost trend (growth, regression, prediction)
pub async fn get_metric_k8s_cluster_cost_trend(
    node_names: Vec<String>,
    unit_prices: InfoUnitPriceEntity,
    q: RangeQuery,
) -> Result<Value> {
    let raw_value = get_metric_k8s_cluster_cost(node_names, unit_prices.clone(), q).await?;
    let cluster_cost: MetricGetResponseDto = serde_json::from_value(raw_value)?;

    let response = build_cost_trend_dto(&cluster_cost, MetricScope::Cluster, None)?;

    Ok(serde_json::to_value(response)?)
}

/// Compute cluster-level resource efficiency (CPU, memory, storage)
pub async fn get_metric_k8s_cluster_raw_efficiency(
    node_info_list: Vec<InfoNodeEntity>,
    node_names: Vec<String>,
    q: RangeQuery,
) -> Result<Value> {
    // 1️⃣ Get summarized usage metrics
    let raw_value = get_metric_k8s_cluster_raw_summary(node_names.clone(), q.clone()).await?;
    let summary: MetricRawSummaryResponseDto = serde_json::from_value(raw_value)?;

    // 2️⃣ Compute total allocatable capacity from node info
    let mut total_cpu_alloc = 0.0;
    let mut total_mem_alloc_bytes = 0.0;
    let mut total_storage_alloc_bytes = 0.0;

    for n in &node_info_list {
        total_cpu_alloc += n.cpu_allocatable_cores.unwrap_or(0) as f64;
        total_mem_alloc_bytes += n.memory_allocatable_bytes.unwrap_or(0) as f64;
        total_storage_alloc_bytes += n.ephemeral_storage_allocatable_bytes.unwrap_or(0) as f64;
    }

    let total_mem_alloc_gb = total_mem_alloc_bytes / 1_073_741_824.0;
    let total_storage_alloc_gb = total_storage_alloc_bytes / 1_073_741_824.0;

    // 3️⃣ Compute efficiency ratios
    let cpu_eff = if total_cpu_alloc > 0.0 {
        (summary.summary.avg_cpu_cores / total_cpu_alloc).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let mem_eff = if total_mem_alloc_gb > 0.0 {
        (summary.summary.avg_memory_gb / total_mem_alloc_gb).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let storage_eff = if total_storage_alloc_gb > 0.0 {
        (summary.summary.avg_storage_gb / total_storage_alloc_gb).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let overall_eff = (cpu_eff + mem_eff + storage_eff) / 3.0;

    // 4️⃣ Build DTO
    let dto = MetricRawEfficiencyResponseDto {
        start: summary.start,
        end: summary.end,
        scope: MetricScope::Cluster,
        granularity: summary.granularity,
        efficiency: MetricRawEfficiencyDto {
            cpu_efficiency: cpu_eff,
            memory_efficiency: mem_eff,
            storage_efficiency: storage_eff,
            overall_efficiency: overall_eff,
            total_cpu_allocatable_cores: total_cpu_alloc,
            total_memory_allocatable_gb: total_mem_alloc_gb,
            total_storage_allocatable_gb: total_storage_alloc_gb,
        },
    };

    Ok(serde_json::to_value(dto)?)
}

#[must_use] // Dropping aggregated data is almost certainly unintended.
pub fn aggregate_cluster_points(
    points: Vec<UniversalMetricPointDto>,
) -> Vec<UniversalMetricPointDto> {
    use std::collections::BTreeMap;

    let mut buckets: BTreeMap<DateTime<Utc>, Vec<UniversalMetricPointDto>> = BTreeMap::new();

    for p in points {
        buckets.entry(p.time).or_default().push(p);
    }

    let mut result = Vec::with_capacity(buckets.len());

    for (time, bucket) in buckets {
        // CPU
        let mut cpu_sum = 0.0;
        let mut cpu_count = 0.0;
        let mut cpu_core_sum = 0.0;
        let mut cpu_core_count = 0.0;

        // Memory
        let mut mem_sum = 0.0;
        let mut mem_count = 0.0;
        let mut mem_working_sum = 0.0;
        let mut mem_working_count = 0.0;
        let mut mem_rss_sum = 0.0;
        let mut mem_rss_count = 0.0;
        let mut mem_pf_sum = 0.0;
        let mut mem_pf_count = 0.0;

        // Filesystem SUM
        let mut fs_used_sum = 0.0;
        let mut fs_capacity_sum = 0.0;

        // Network SUM
        let mut rx_sum = 0.0;
        let mut tx_sum = 0.0;
        let mut rx_err_sum = 0.0;
        let mut tx_err_sum = 0.0;

        for p in &bucket {
            // CPU AVG
            if let Some(v) = p.cpu_memory.cpu_usage_nano_cores {
                cpu_sum += v;
                cpu_count += 1.0;
            }
            if let Some(v) = p.cpu_memory.cpu_usage_core_nano_seconds {
                cpu_core_sum += v;
                cpu_core_count += 1.0;
            }
            // MEMORY AVG
            if let Some(v) = p.cpu_memory.memory_usage_bytes {
                mem_sum += v;
                mem_count += 1.0;
            }
            if let Some(v) = p.cpu_memory.memory_working_set_bytes {
                mem_working_sum += v;
                mem_working_count += 1.0;
            }
            if let Some(v) = p.cpu_memory.memory_rss_bytes {
                mem_rss_sum += v;
                mem_rss_count += 1.0;
            }
            if let Some(v) = p.cpu_memory.memory_page_faults {
                mem_pf_sum += v;
                mem_pf_count += 1.0;
            }

            // FILESYSTEM SUM
            if let Some(fs) = &p.filesystem {
                fs_used_sum += fs.used_bytes.unwrap_or(0.0);
                fs_capacity_sum += fs.capacity_bytes.unwrap_or(0.0);
            }

            // NETWORK SUM
            if let Some(net) = &p.network {
                rx_sum += net.rx_bytes.unwrap_or(0.0);
                tx_sum += net.tx_bytes.unwrap_or(0.0);
                rx_err_sum += net.rx_errors.unwrap_or(0.0);
                tx_err_sum += net.tx_errors.unwrap_or(0.0);
            }
        }

        result.push(UniversalMetricPointDto {
            time,
            cpu_memory: CommonMetricValuesDto {
                cpu_usage_nano_cores: (cpu_count > 0.0).then(|| cpu_sum / cpu_count),
                cpu_usage_core_nano_seconds: (cpu_core_count > 0.0)
                    .then(|| cpu_core_sum / cpu_core_count),
                memory_usage_bytes: (mem_count > 0.0).then(|| mem_sum / mem_count),
                memory_working_set_bytes: (mem_working_count > 0.0)
                    .then(|| mem_working_sum / mem_working_count),
                memory_rss_bytes: (mem_rss_count > 0.0)
                    .then(|| mem_rss_sum / mem_rss_count),
                memory_page_faults: (mem_pf_count > 0.0)
                    .then(|| mem_pf_sum / mem_pf_count),
            },
            filesystem: Some(FilesystemMetricDto {
                used_bytes: Some(fs_used_sum),
                capacity_bytes: Some(fs_capacity_sum),
                inodes_used: None,
                inodes: None,
            }),
            network: Some(NetworkMetricDto {
                rx_bytes: Some(rx_sum),
                tx_bytes: Some(tx_sum),
                rx_errors: Some(rx_err_sum),
                tx_errors: Some(tx_err_sum),
            }),
            storage: None,
            cost: None,
        });
    }

    result
}



