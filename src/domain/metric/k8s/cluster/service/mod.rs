use crate::api::dto::metrics_dto::RangeQuery;
use crate::core::persistence::info::fixed::unit_price::info_unit_price_entity::InfoUnitPriceEntity;
use crate::core::persistence::info::k8s::node::info_node_entity::InfoNodeEntity;
use crate::core::persistence::metrics::k8s::node::day::metric_node_day_api_repository_trait::MetricNodeDayApiRepository;
use crate::core::persistence::metrics::k8s::node::hour::metric_node_hour_api_repository_trait::MetricNodeHourApiRepository;
use crate::core::persistence::metrics::k8s::node::minute::metric_node_minute_api_repository_trait::MetricNodeMinuteApiRepository;
use crate::domain::metric::k8s::common::dto::metric_k8s_cost_summary_dto::{MetricCostSummaryDto, MetricCostSummaryResponseDto};
use crate::domain::metric::k8s::common::dto::metric_k8s_cost_trend_dto::{MetricCostTrendDto, MetricCostTrendPointDto, MetricCostTrendResponseDto};
use crate::domain::metric::k8s::common::dto::metric_k8s_raw_efficiency_dto::{MetricRawEfficiencyDto, MetricRawEfficiencyResponseDto};
use crate::domain::metric::k8s::common::dto::metric_k8s_raw_summary_dto::{MetricRawSummaryDto, MetricRawSummaryResponseDto};
use crate::domain::metric::k8s::common::dto::{CommonMetricValuesDto, CostMetricDto, FilesystemMetricDto, MetricGetResponseDto, MetricScope, MetricSeriesDto, NetworkMetricDto, UniversalMetricPointDto};
use crate::domain::metric::k8s::common::service_helpers::resolve_time_window;
use crate::domain::metric::k8s::common::util::k8s_metric_repository_resolve::resolve_k8s_metric_repository;
use crate::domain::metric::k8s::common::util::k8s_metric_repository_variant::K8sMetricRepositoryVariant;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;

pub async fn get_metric_k8s_cluster_raw(
    node_info_list: Vec<InfoNodeEntity>,
    q: RangeQuery,
) -> Result<Value, anyhow::Error> {

    let window = resolve_time_window(&q);
    let repo = resolve_k8s_metric_repository(&MetricScope::Node, &window.granularity);

    let mut aggregated_points: Vec<UniversalMetricPointDto> = Vec::new();

    for node in &node_info_list {
        let Some(node_name) = &node.node_name else {
            continue; // skip invalid node entries
        };

        // Load per-node metric rows
        let rows = match &repo {
            K8sMetricRepositoryVariant::NodeMinute(r) =>
                r.get_row_between(node_name, window.start, window.end),
            K8sMetricRepositoryVariant::NodeHour(r) =>
                r.get_row_between(node_name, window.start, window.end),
            K8sMetricRepositoryVariant::NodeDay(r) =>
                r.get_row_between(node_name, window.start, window.end),
            _ => Ok(vec![]),
        }.unwrap_or_else(|err| {
            tracing::warn!("Failed loading metrics for {}: {}", node_name, err);
            vec![]
        });

        // Convert to universal struct — preserve missing values (None/null)
        aggregated_points.extend(rows.into_iter().map(|m| {
            UniversalMetricPointDto {
                time: m.time,
                cpu_memory: CommonMetricValuesDto {
                    cpu_usage_nano_cores: m.cpu_usage_nano_cores.map(|v| v as f64),
                    memory_usage_bytes: m.memory_usage_bytes.map(|v| v as f64),
                    memory_working_set_bytes: m.memory_working_set_bytes.map(|v| v as f64),
                    memory_rss_bytes: m.memory_rss_bytes.map(|v| v as f64),
                    memory_page_faults: m.memory_page_faults.map(|v| v as f64),
                    ..Default::default()
                },
                filesystem: Some(FilesystemMetricDto {
                    used_bytes: m.fs_used_bytes.map(|v| v as f64),
                    capacity_bytes: m.fs_capacity_bytes.map(|v| v as f64),
                    inodes_used: m.fs_inodes_used.map(|v| v as f64),
                    inodes: m.fs_inodes.map(|v| v as f64),
                    ..Default::default()
                }),
                network: Some(NetworkMetricDto {
                    rx_bytes: m.network_physical_rx_bytes.map(|v| v as f64),
                    tx_bytes: m.network_physical_tx_bytes.map(|v| v as f64),
                    rx_errors: m.network_physical_rx_errors.map(|v| v as f64),
                    tx_errors: m.network_physical_tx_errors.map(|v| v as f64),
                    ..Default::default()
                }),
                ..Default::default()
            }
        }));
    }

    // Aggregate multiple nodes → cluster values
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
        }],
        // Cluster API does not paginate output
        total: None,
        limit: None,
        offset: None,
    };

    Ok(serde_json::to_value(response)?)
}


/// Summarize raw cluster resource usage (CPU, memory, storage, network)
pub async fn get_metric_k8s_cluster_raw_summary(
    node_info_list: Vec<InfoNodeEntity>,
    q: RangeQuery,
) -> Result<Value> {
    // 1️⃣ Retrieve the raw metrics for the time range
    let raw_value = get_metric_k8s_cluster_raw(node_info_list.clone(), q.clone()).await?;
    let cluster_metrics: MetricGetResponseDto = serde_json::from_value(raw_value)?;

    // 2️⃣ Prepare accumulators
    let mut total_cpu = 0.0;
    let mut max_cpu = 0.0;
    let mut total_mem = 0.0;
    let mut max_mem = 0.0;
    let mut total_storage = 0.0;
    let mut max_storage = 0.0;
    let mut total_network = 0.0;
    let mut max_network = 0.0;
    let mut point_count = 0.0;

    // 3️⃣ Aggregate usage across all metric points
    for series in &cluster_metrics.series {
        for point in &series.points {
            let cpu = point.cpu_memory.cpu_usage_nano_cores.unwrap_or(0.0) / 1_000_000_000.0; // nanocores → cores
            let mem_gb = point.cpu_memory.memory_usage_bytes.unwrap_or(0.0) / 1_073_741_824.0; // bytes → GB
            let fs_gb = point
                .filesystem
                .as_ref()
                .and_then(|f| f.used_bytes)
                .unwrap_or(0.0) / 1_073_741_824.0;
            let net_gb = point
                .network
                .as_ref()
                .map(|n| {
                    (n.rx_bytes.unwrap_or(0.0) + n.tx_bytes.unwrap_or(0.0))
                        / 1_073_741_824.0
                })
                .unwrap_or(0.0);

            total_cpu += cpu;
            total_mem += mem_gb;
            total_storage += fs_gb;
            total_network += net_gb;

            if cpu > max_cpu { max_cpu = cpu; }
            if mem_gb > max_mem { max_mem = mem_gb; }
            if fs_gb > max_storage { max_storage = fs_gb; }
            if net_gb > max_network { max_network = net_gb; }

            point_count += 1.0;
        }
    }

    if point_count == 0.0 {
        return Ok(serde_json::json!({ "status": "no data" }));
    }

    // 4️⃣ Compute averages
    let summary = MetricRawSummaryDto {
        avg_cpu_cores: total_cpu / point_count,
        max_cpu_cores: max_cpu,
        avg_memory_gb: total_mem / point_count,
        max_memory_gb: max_mem,
        avg_storage_gb: total_storage / point_count,
        max_storage_gb: max_storage,
        avg_network_gb: total_network / point_count,
        max_network_gb: max_network,
        node_count: node_info_list.len(),
    };

    // 5️⃣ Wrap in response DTO
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
    node_info_list: Vec<InfoNodeEntity>,
    unit_prices: InfoUnitPriceEntity,
    q: RangeQuery,
) -> Result<Value> {
    // 1️⃣ Get raw cluster metrics first
    let raw_value = get_metric_k8s_cluster_raw(node_info_list, q).await?;
    let mut resp: MetricGetResponseDto = serde_json::from_value(raw_value)?;

    // 2️⃣ Compute cost per metric point
    for series in &mut resp.series {
        for point in &mut series.points {
            // --- CPU cost ---
            let cpu_cost_usd = point
                .cpu_memory
                .cpu_usage_nano_cores
                .map(|nano| {
                    let cores = nano / 1_000_000_000.0; // convert nanocores to cores
                    // Convert hourly price → per-second
                    cores * (unit_prices.cpu_core_hour / 3600.0)
                });

            // --- Memory cost ---
            let memory_cost_usd = point
                .cpu_memory
                .memory_usage_bytes
                .map(|bytes| {
                    let gb = bytes / (1024.0 * 1024.0 * 1024.0); // bytes → GB
                    gb * (unit_prices.memory_gb_hour / 3600.0)
                });

            // --- Storage cost ---
            let storage_cost_usd = point
                .filesystem
                .as_ref()
                .and_then(|fs| fs.used_bytes)
                .map(|bytes| {
                    let gb = bytes / (1024.0 * 1024.0 * 1024.0);
                    gb * (unit_prices.storage_gb_hour / 3600.0)
                });

            // --- Sum up total ---
            let total_cost_usd = Some(
                cpu_cost_usd.unwrap_or(0.0)
                    + memory_cost_usd.unwrap_or(0.0)
                    + storage_cost_usd.unwrap_or(0.0),
            );

            // --- Store in cost field ---
            point.cost = Some(CostMetricDto {
                total_cost_usd,
                cpu_cost_usd,
                memory_cost_usd,
                storage_cost_usd,
            });
        }
    }

    // 3️⃣ Serialize response back to JSON
    Ok(serde_json::to_value(resp)?)
}


/// Summarize total cluster cost across all time points and resources
pub async fn get_metric_k8s_cluster_cost_summary(
    node_info_list: Vec<InfoNodeEntity>,
    unit_prices: InfoUnitPriceEntity,
    q: RangeQuery,
) -> Result<Value> {
    // 1️⃣ Get detailed cluster cost metrics
    let raw_value = get_metric_k8s_cluster_cost(node_info_list, unit_prices.clone(), q).await?;
    let cluster_cost: MetricGetResponseDto = serde_json::from_value(raw_value)?;

    // 2️⃣ Aggregate totals
    let mut summary = MetricCostSummaryDto::default();

    for series in cluster_cost.series {
        for point in series.points {
            if let Some(c) = point.cost {
                summary.cpu_cost_usd += c.cpu_cost_usd.unwrap_or(0.0);
                summary.memory_cost_usd += c.memory_cost_usd.unwrap_or(0.0);

                // Split storage cost into ephemeral + persistent if available
                let ephemeral_cost = point
                    .filesystem
                    .as_ref()
                    .and_then(|fs| fs.used_bytes)
                    .map(|b| b / (1024.0 * 1024.0 * 1024.0) * unit_prices.storage_gb_hour / 3600.0)
                    .unwrap_or(0.0);

                let persistent_cost = point
                    .storage
                    .as_ref()
                    .and_then(|s| s.persistent.as_ref())
                    .and_then(|fs| fs.used_bytes)
                    .map(|b| b / (1024.0 * 1024.0 * 1024.0) * unit_prices.storage_gb_hour / 3600.0)
                    .unwrap_or(0.0);

                let network_cost = point
                    .network
                    .as_ref()
                    .map(|n| {
                        let rx_gb = n.rx_bytes.unwrap_or(0.0) / 1_073_741_824.0;
                        let tx_gb = n.tx_bytes.unwrap_or(0.0) / 1_073_741_824.0;
                        // Simplified: treat all traffic as external
                        (rx_gb + tx_gb) * unit_prices.network_external_gb / 3600.0
                    })
                    .unwrap_or(0.0);

                summary.ephemeral_storage_cost_usd += ephemeral_cost;
                summary.persistent_storage_cost_usd += persistent_cost;
                summary.network_cost_usd += network_cost;

                summary.total_cost_usd += c.total_cost_usd.unwrap_or(0.0)
                    + ephemeral_cost
                    + persistent_cost
                    + network_cost;
            }
        }
    }

    // 3️⃣ Build and serialize DTO
    let summary_dto = MetricCostSummaryResponseDto {
        start: cluster_cost.start,
        end: cluster_cost.end,
        scope: MetricScope::Cluster,
        target: None,
        granularity: cluster_cost.granularity,
        summary,
    };

    Ok(serde_json::to_value(summary_dto)?)
}


/// Analyze cluster cost trend (growth, regression, prediction)
/// Analyze cluster cost trend (growth, regression, prediction)
pub async fn get_metric_k8s_cluster_cost_trend(
    node_info_list: Vec<InfoNodeEntity>,
    unit_prices: InfoUnitPriceEntity,
    q: RangeQuery,
) -> Result<Value> {

    // 1️⃣ Fetch cost-enriched metrics
    let raw_value = get_metric_k8s_cluster_cost(node_info_list, unit_prices.clone(), q).await?;
    let cluster_cost: MetricGetResponseDto = serde_json::from_value(raw_value)?;

    // 2️⃣ Flatten all points into a unified list
    let mut cost_points: Vec<MetricCostTrendPointDto> = Vec::new();

    for series in &cluster_cost.series {
        for point in &series.points {
            if let Some(cost) = &point.cost {
                if let Some(total) = cost.total_cost_usd {
                    cost_points.push(MetricCostTrendPointDto {
                        time: point.time,
                        total_cost_usd: total,
                        cpu_cost_usd: cost.cpu_cost_usd.unwrap_or(0.0),
                        memory_cost_usd: cost.memory_cost_usd.unwrap_or(0.0),
                        storage_cost_usd: cost.storage_cost_usd.unwrap_or(0.0),
                    });
                }
            }
        }
    }

    // 3️⃣ Sort by timestamp
    cost_points.sort_by_key(|p| p.time);

    if cost_points.is_empty() {
        return Ok(serde_json::json!({
            "error": "no cost data available for trend analysis"
        }));
    }

    // 4️⃣ Trend statistics
    let start_cost = cost_points.first().unwrap().total_cost_usd;
    let end_cost = cost_points.last().unwrap().total_cost_usd;
    let diff = end_cost - start_cost;

    let growth_rate = if start_cost > 0.0 {
        (diff / start_cost) * 100.0
    } else {
        0.0
    };

    // 5️⃣ Linear regression using real timestamps as X-axis
    // Convert timestamps to seconds since epoch (f64)
    let xs: Vec<f64> = cost_points.iter().map(|p| p.time.timestamp() as f64).collect();
    let ys: Vec<f64> = cost_points.iter().map(|p| p.total_cost_usd).collect();

    let n = xs.len() as f64;
    let sum_x: f64 = xs.iter().sum();
    let sum_y: f64 = ys.iter().sum();
    let sum_xx: f64 = xs.iter().map(|x| x * x).sum();
    let sum_xy: f64 = xs.iter().zip(ys.iter()).map(|(x, y)| x * y).sum();

    let denominator = n * sum_xx - sum_x * sum_x;

    let slope = if denominator.abs() > f64::EPSILON {
        (n * sum_xy - sum_x * sum_y) / denominator
    } else {
        0.0
    };

    // 6️⃣ Predict next point (using last timestamp + granularity)
    let predicted_next = {
        let last_time = cost_points.last().unwrap().time.timestamp() as f64;
        Some(end_cost + slope * (last_time + 1.0 - last_time)) // simple +1 step
    };

    // 7️⃣ Build response DTO
    let response = MetricCostTrendResponseDto {
        start: cluster_cost.start,
        end: cluster_cost.end,
        scope: MetricScope::Cluster,
        target: None,
        granularity: cluster_cost.granularity,
        trend: MetricCostTrendDto {
            start_cost_usd: start_cost,
            end_cost_usd: end_cost,
            cost_diff_usd: diff,
            growth_rate_percent: growth_rate,
            regression_slope_usd_per_granularity: slope,
            predicted_next_cost_usd: predicted_next,
        },
        points: cost_points,
    };

    Ok(serde_json::to_value(response)?)
}


/// Compute cluster-level resource efficiency (CPU, memory, storage)
pub async fn get_metric_k8s_cluster_raw_efficiency(
    node_info_list: Vec<InfoNodeEntity>,
    q: RangeQuery,
) -> Result<Value> {
    // 1️⃣ Get summarized usage metrics
    let raw_value = get_metric_k8s_cluster_raw_summary(node_info_list.clone(), q.clone()).await?;
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
                ..Default::default()
            },
            filesystem: Some(FilesystemMetricDto {
                used_bytes: Some(fs_used_sum),
                capacity_bytes: Some(fs_capacity_sum),
                ..Default::default()
            }),
            network: Some(NetworkMetricDto {
                rx_bytes: Some(rx_sum),
                tx_bytes: Some(tx_sum),
                rx_errors: Some(rx_err_sum),
                tx_errors: Some(tx_err_sum),
                ..Default::default()
            }),
            ..Default::default()
        });
    }

    result
}
