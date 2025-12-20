use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use crate::api::dto::metrics_dto::RangeQuery;
use crate::core::persistence::info::fixed::unit_price::info_unit_price_entity::InfoUnitPriceEntity;
use crate::domain::metric::k8s::common::dto::{
    CommonMetricValuesDto, CostMetricDto, FilesystemMetricDto, MetricGetResponseDto, MetricGranularity,
    MetricScope, MetricSeriesDto, UniversalMetricPointDto,
};
use crate::domain::metric::k8s::common::dto::metric_k8s_cost_summary_dto::{
    MetricCostSummaryDto, MetricCostSummaryResponseDto,
};
use crate::domain::metric::k8s::common::dto::metric_k8s_cost_trend_dto::{MetricCostTrendDto, MetricCostTrendPointDto, MetricCostTrendResponseDto};
use crate::domain::metric::k8s::common::dto::metric_k8s_raw_efficiency_dto::{
    MetricRawEfficiencyDto, MetricRawEfficiencyResponseDto,
};
use crate::domain::metric::k8s::common::dto::metric_k8s_raw_summary_dto::{
    MetricRawSummaryDto, MetricRawSummaryResponseDto,
};
use crate::domain::metric::k8s::common::util::k8s_metric_determine_granularity::determine_granularity;
use std::collections::HashMap;
use tracing::log::warn;
use crate::core::persistence::info::k8s::node::info_node_entity::InfoNodeEntity;
use crate::core::util::cost_util::CostUtil;

pub const BYTES_PER_GB: f64 = 1_073_741_824.0;

#[derive(Clone)]
pub struct TimeWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub granularity: MetricGranularity,
}

pub fn resolve_time_window(q: &RangeQuery) -> TimeWindow {
    let start = q
        .start
        .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or_else(|| Utc::now() - chrono::Duration::hours(1));

    let end = q
        .end
        .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or_else(Utc::now);

    let granularity = match q.granularity.clone() {
        Some(g) => {
            // Soft validation: log but never fail
            if let Err(err) = validate_granularity(start, end, g.clone()) {
                warn!("Invalid granularity override {:?}: {}", g, err);
                // fallback to automatic granularity
                determine_granularity(start, end)
            } else {
                g
            }
        }
        None => determine_granularity(start, end),
    };

    TimeWindow { start, end, granularity }
}


pub fn validate_granularity(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    granularity: MetricGranularity,
) -> Result<(), String> {
    let diff = end - start;

    match granularity {
        MetricGranularity::Minute => {
            if diff > chrono::Duration::hours(3) {
                return Err("minute granularity cannot be used for ranges > 3 hours".into());
            }
        }
        MetricGranularity::Hour => {
            if diff > chrono::Duration::days(3) {
                return Err("hour granularity cannot be used for ranges > 3 days".into());
            }
        }
        MetricGranularity::Day => { /* always allowed */ }
    }

    Ok(())
}

pub fn build_raw_summary_value(
    metrics: &MetricGetResponseDto,
    scope: MetricScope,
    member_count: usize,
) -> Result<Value> {
    let mut total_cpu = 0.0;
    let mut max_cpu = 0.0;
    let mut total_mem = 0.0;
    let mut max_mem = 0.0;
    let mut total_storage = 0.0;
    let mut max_storage = 0.0;
    let mut total_network = 0.0;
    let mut max_network = 0.0;
    let mut point_count = 0.0;

    for series in &metrics.series {
        for point in &series.points {
            let cpu = point.cpu_memory.cpu_usage_nano_cores.unwrap_or(0.0) / 1_000_000_000.0;
            let mem_gb = point.cpu_memory.memory_usage_bytes.unwrap_or(0.0) / BYTES_PER_GB;
            let fs_gb = point
                .filesystem
                .as_ref()
                .and_then(|fs| fs.used_bytes)
                .unwrap_or(0.0)
                / BYTES_PER_GB;
            let net_gb = point
                .network
                .as_ref()
                .map(|n| {
                    (n.rx_bytes.unwrap_or(0.0) + n.tx_bytes.unwrap_or(0.0)) / BYTES_PER_GB
                })
                .unwrap_or(0.0);

            total_cpu += cpu;
            total_mem += mem_gb;
            total_storage += fs_gb;
            total_network += net_gb;

            if cpu > max_cpu {
                max_cpu = cpu;
            }
            if mem_gb > max_mem {
                max_mem = mem_gb;
            }
            if fs_gb > max_storage {
                max_storage = fs_gb;
            }
            if net_gb > max_network {
                max_network = net_gb;
            }

            point_count += 1.0;
        }
    }

    if point_count == 0.0 {
        return Ok(json!({ "status": "no data" }));
    }

    let summary = MetricRawSummaryDto {
        avg_cpu_cores: total_cpu / point_count,
        max_cpu_cores: max_cpu,
        avg_memory_gb: total_mem / point_count,
        max_memory_gb: max_mem,
        avg_storage_gb: total_storage / point_count,
        max_storage_gb: max_storage,
        avg_network_gb: total_network / point_count,
        max_network_gb: max_network,
        node_count: member_count,
    };

    let dto = MetricRawSummaryResponseDto {
        start: metrics.start,
        end: metrics.end,
        scope,
        granularity: metrics.granularity.clone(),
        summary,
    };

    Ok(serde_json::to_value(dto)?)
}

fn granularity_interval_hours(granularity: &MetricGranularity) -> f64 {
    match granularity {
        MetricGranularity::Minute => 1.0 / 60.0,
        MetricGranularity::Hour => 1.0,
        MetricGranularity::Day => 24.0,
    }
}

fn point_interval_hours(points: &[UniversalMetricPointDto], idx: usize, default: f64) -> f64 {
    if let Some(next) = points.get(idx + 1) {
        let delta_seconds = next.time.signed_duration_since(points[idx].time).num_seconds();
        if delta_seconds > 0 {
            return delta_seconds as f64 / 3600.0;
        }
    }
    default
}
fn point_interval_hours_from_timestamps(
    timestamps: &[DateTime<Utc>],
    idx: usize,
    default_interval_hours: f64,
) -> f64 {
    if timestamps.len() < 2 {
        return default_interval_hours;
    }

    // First point → interval = next - current
    if idx == 0 {
        return (timestamps[1] - timestamps[0]).num_seconds() as f64 / 3600.0;
    }

    // Otherwise use difference between this point and previous point
    let seconds = (timestamps[idx] - timestamps[idx - 1]).num_seconds();

    if seconds > 0 {
        seconds as f64 / 3600.0
    } else {
        // Fallback for clock drift, identical timestamps, errors, etc
        default_interval_hours
    }
}
pub fn apply_costs(response: &mut MetricGetResponseDto, unit_prices: &InfoUnitPriceEntity) {
    let default_interval_hours = granularity_interval_hours(&response.granularity);

    for series in &mut response.series {
        // Precompute timestamps (avoids borrow conflicts)
        let timestamps: Vec<_> = series.points.iter().map(|p| p.time).collect();

        for (idx, point) in series.points.iter_mut().enumerate() {
            let interval_hours =
                point_interval_hours_from_timestamps(&timestamps, idx, default_interval_hours);

            // ---------------------------
            // CPU (usage-based)
            // ---------------------------
            // IMPORTANT:
            // - cpu_usage_nano_cores is a gauge (instantaneous), suitable for graphs, not cost.
            // - cpu_usage_core_nano_seconds should already represent "usage within the interval"
            //   after minute->hour (increase) and hour->day (sum).
            let cpu_cost_usd = point.cpu_memory.cpu_usage_core_nano_seconds
                .map(|core_nano_seconds| {
                    CostUtil::compute_cpu_cost_from_core_nano_seconds(core_nano_seconds, unit_prices)
                });

            // ---------------------------
            // MEMORY (gauge * time)
            // ---------------------------
            // Prefer working_set for cost, fallback to memory_usage if missing.
            let memory_bytes_for_cost = point.cpu_memory.memory_working_set_bytes
                .or(point.cpu_memory.memory_usage_bytes);

            let memory_cost_usd = memory_bytes_for_cost
                .map(|bytes| CostUtil::compute_memory_cost(bytes, interval_hours, unit_prices));

            // ---------------------------
            // STORAGE (gauge * time)
            // ---------------------------
            let ephemeral_gb_hours = point.filesystem
                .as_ref()
                .and_then(|fs| fs.used_bytes)
                .map(|b| CostUtil::bytes_to_gb_hours(b, interval_hours))
                .unwrap_or(0.0);

            let persistent_gb_hours = point.storage
                .as_ref()
                .and_then(|s| s.persistent.as_ref())
                .and_then(|fs| fs.used_bytes)
                .map(|b| CostUtil::bytes_to_gb_hours(b, interval_hours))
                .unwrap_or(0.0);

            let total_storage_gb_hours = ephemeral_gb_hours + persistent_gb_hours;
            let storage_cost_usd = Some(total_storage_gb_hours * unit_prices.storage_gb_hour);

            // ---------------------------
            // NETWORK (usage-based)
            // ---------------------------
            // If rx/tx are interval usage (bytes), do NOT multiply by interval_hours.
            let network_cost_usd: f64 = point.network.as_ref().map(|n| {
                let rx_gb = CostUtil::bytes_to_gb(n.rx_bytes.unwrap_or(0.0));
                let tx_gb = CostUtil::bytes_to_gb(n.tx_bytes.unwrap_or(0.0));
                (rx_gb + tx_gb) * unit_prices.network_external_gb
            }).unwrap_or(0.0);

            // ---------------------------
            // TOTAL
            // ---------------------------
            let total_cost_usd = Some(
                cpu_cost_usd.unwrap_or(0.0)
                    + memory_cost_usd.unwrap_or(0.0)
                    + storage_cost_usd.unwrap_or(0.0)
                    + network_cost_usd
            );

            point.cost = Some(CostMetricDto {
                total_cost_usd,
                cpu_cost_usd,
                memory_cost_usd,
                storage_cost_usd,
            });
        }
    }
}

pub fn apply_node_costs(
    response: &mut MetricGetResponseDto,
    unit_prices: &InfoUnitPriceEntity,
    node_infos: &Vec<InfoNodeEntity>,
) {
    for series in &mut response.series {
        // 🔹 series.key == node_name
        let node_name = &series.key;

        // Find Node Info
        let node_info = match node_infos.iter().find(|n| n.node_name.as_deref() == Some(node_name)) {
            Some(n) => n,
            None => continue,
        };

        // Check Running Hours
        let running_hours = match series.running_hours {
            Some(h) if h > 0.0 => h,
            _ => continue,
        };

        // Get Resource Capacity
        let cpu_cores = node_info.cpu_capacity_cores.unwrap_or(0) as f64;
        let memory_gb =
            node_info.memory_capacity_bytes.unwrap_or(0) as f64 / 1_073_741_824.0;
        let storage_gb =
            node_info.ephemeral_storage_capacity_bytes.unwrap_or(0) as f64 / 1_073_741_824.0;


        let cpu_cost_usd = Some(cpu_cores * running_hours * unit_prices.cpu_core_hour);
        let memory_cost_usd = Some(memory_gb * running_hours * unit_prices.memory_gb_hour);
        let storage_cost_usd = Some(storage_gb * running_hours * unit_prices.storage_gb_hour);

        let network_cost_usd = 0.0;

        let total_cost_usd = Some(
            cpu_cost_usd.unwrap_or(0.0)
                + memory_cost_usd.unwrap_or(0.0)
                + storage_cost_usd.unwrap_or(0.0)
                + network_cost_usd
        );

        series.cost_summary = Some(CostMetricDto {
            total_cost_usd,
            cpu_cost_usd,
            memory_cost_usd,
            storage_cost_usd,
        });
    }
}


pub fn build_cost_summary_dto(
    metrics: &MetricGetResponseDto,
    scope: MetricScope,
    target: Option<String>,
    unit_prices: &InfoUnitPriceEntity,
) -> MetricCostSummaryResponseDto {
    let mut summary = MetricCostSummaryDto::default();
    let default_interval_hours = granularity_interval_hours(&metrics.granularity);

    for series in &metrics.series {
        for (idx, point) in series.points.iter().enumerate() {
            let interval_hours = point_interval_hours(&series.points, idx, default_interval_hours);

            if let Some(cost) = &point.cost {
                let cpu_cost = cost.cpu_cost_usd.unwrap_or(0.0);
                let memory_cost = cost.memory_cost_usd.unwrap_or(0.0);

                let ephemeral_cost = point
                    .filesystem
                    .as_ref()
                    .and_then(|fs| fs.used_bytes)
                    .map(|b| (b / BYTES_PER_GB) * interval_hours * unit_prices.storage_gb_hour)
                    .unwrap_or(0.0);

                let persistent_cost = point
                    .storage
                    .as_ref()
                    .and_then(|s| s.persistent.as_ref())
                    .and_then(|fs| fs.used_bytes)
                    .map(|b| (b / BYTES_PER_GB) * interval_hours * unit_prices.storage_gb_hour)
                    .unwrap_or(0.0);

                let network_cost = point
                    .network
                    .as_ref()
                    .map(|n| {
                        let rx_gb = n.rx_bytes.unwrap_or(0.0) / BYTES_PER_GB;
                        let tx_gb = n.tx_bytes.unwrap_or(0.0) / BYTES_PER_GB;
                        (rx_gb + tx_gb) * unit_prices.network_external_gb
                    })
                    .unwrap_or(0.0);

                summary.cpu_cost_usd += cpu_cost;
                summary.memory_cost_usd += memory_cost;
                summary.ephemeral_storage_cost_usd += ephemeral_cost;
                summary.persistent_storage_cost_usd += persistent_cost;
                summary.network_cost_usd += network_cost;

                summary.total_cost_usd += cpu_cost + memory_cost + ephemeral_cost + persistent_cost + network_cost;
            }
        }
    }

    MetricCostSummaryResponseDto {
        start: metrics.start,
        end: metrics.end,
        scope,
        target,
        granularity: metrics.granularity.clone(),
        summary,
    }
}

pub fn build_node_cost_summary_dto(
    metrics: &MetricGetResponseDto,
    scope: MetricScope,
    target: Option<String>,
    unit_prices: &InfoUnitPriceEntity,
) -> MetricCostSummaryResponseDto {
    let mut summary = MetricCostSummaryDto::default();

    for series in &metrics.series {
        for (idx, point) in series.points.iter().enumerate() {
            let mut network_cost = 0.0;
            if let Some(cost) = &point.cost {
                network_cost = point
                    .network
                    .as_ref()
                    .map(|n| {
                        let rx_gb = n.rx_bytes.unwrap_or(0.0) / BYTES_PER_GB;
                        let tx_gb = n.tx_bytes.unwrap_or(0.0) / BYTES_PER_GB;
                        (rx_gb + tx_gb) * unit_prices.network_external_gb
                    })
                    .unwrap_or(0.0);


            }
            summary.network_cost_usd += network_cost;


        }
        summary.cpu_cost_usd += series.cost_summary.as_ref().map(|c| c.cpu_cost_usd.unwrap_or(0.0)).unwrap_or(0.0);
        summary.memory_cost_usd += series.cost_summary.as_ref().map(|c| c.memory_cost_usd.unwrap_or(0.0)).unwrap_or(0.0);
        summary.ephemeral_storage_cost_usd += series.cost_summary.as_ref().map(|c| c.storage_cost_usd.unwrap_or(0.0)).unwrap_or(0.0);
        summary.total_cost_usd += series.cost_summary.as_ref().map(|c| c.total_cost_usd.unwrap_or(0.0)).unwrap_or(0.0) + summary.network_cost_usd;
    }

    MetricCostSummaryResponseDto {
        start: metrics.start,
        end: metrics.end,
        scope,
        target,
        granularity: metrics.granularity.clone(),
        summary,
    }
}

pub fn build_cost_trend_dto(
    metrics: &MetricGetResponseDto,
    scope: MetricScope,
    target: Option<String>,
) -> Result<MetricCostTrendResponseDto> {

    // 1️⃣ Extract all cost points into flattened trend points
    let mut trend_points: Vec<MetricCostTrendPointDto> = metrics
        .series
        .iter()
        .flat_map(|series| {
            series.points.iter().filter_map(|p| {
                p.cost.as_ref().and_then(|c| {
                    c.total_cost_usd.map(|total| MetricCostTrendPointDto {
                        time: p.time,
                        total_cost_usd: total,
                        cpu_cost_usd: c.cpu_cost_usd.unwrap_or(0.0),
                        memory_cost_usd: c.memory_cost_usd.unwrap_or(0.0),
                        storage_cost_usd: c.storage_cost_usd.unwrap_or(0.0),
                    })
                })
            })
        })
        .collect();

    if trend_points.is_empty() {
        return Err(anyhow!("no cost data available for trend analysis"));
    }

    // 2️⃣ Sort by timestamp
    trend_points.sort_by_key(|p| p.time);

    // 3️⃣ Start/end cost
    let start_cost = trend_points.first().unwrap().total_cost_usd;
    let end_cost = trend_points.last().unwrap().total_cost_usd;
    let diff = end_cost - start_cost;

    let growth_rate_percent = if start_cost > 0.0 {
        (diff / start_cost) * 100.0
    } else {
        0.0
    };

    // 4️⃣ Auto regression using UNIX timestamps
    let xs: Vec<f64> = trend_points
        .iter()
        .map(|p| p.time.timestamp() as f64)
        .collect();

    let ys: Vec<f64> = trend_points
        .iter()
        .map(|p| p.total_cost_usd)
        .collect();

    let n = xs.len() as f64;

    let sum_x = xs.iter().sum::<f64>();
    let sum_y = ys.iter().sum::<f64>();
    let sum_xx = xs.iter().map(|x| x * x).sum::<f64>();
    let sum_xy = xs.iter().zip(ys.iter()).map(|(x, y)| x * y).sum::<f64>();

    let denom = n * sum_xx - sum_x * sum_x;

    let slope = if denom.abs() > f64::EPSILON {
        (n * sum_xy - sum_x * sum_y) / denom
    } else {
        0.0
    };

    // Predict the next point — simple linear extrapolation
    let last_x = xs.last().copied().unwrap_or(0.0);
    let predicted_next_cost_usd = Some(end_cost + slope * (last_x + 1.0 - last_x));

    Ok(MetricCostTrendResponseDto {
        start: metrics.start,
        end: metrics.end,
        scope,
        target,
        granularity: metrics.granularity.clone(),

        trend: MetricCostTrendDto {
            start_cost_usd: start_cost,
            end_cost_usd: end_cost,
            cost_diff_usd: diff,
            growth_rate_percent,
            regression_slope_usd_per_granularity: slope,
            predicted_next_cost_usd,
        },

        points: trend_points,
    })
}


pub fn build_efficiency_value(
    summary: MetricRawSummaryResponseDto,
    scope: MetricScope,
    total_cpu_alloc: f64,
    total_mem_alloc_gb: f64,
    total_storage_alloc_gb: f64,
) -> Result<Value> {
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

    let dto = MetricRawEfficiencyResponseDto {
        start: summary.start,
        end: summary.end,
        scope,
        granularity: summary.granularity,
        efficiency: MetricRawEfficiencyDto {
            cpu_efficiency: cpu_eff,
            memory_efficiency: mem_eff,
            storage_efficiency: storage_eff,
            overall_efficiency: (cpu_eff + mem_eff + storage_eff) / 3.0,
            total_cpu_allocatable_cores: total_cpu_alloc,
            total_memory_allocatable_gb: total_mem_alloc_gb,
            total_storage_allocatable_gb: total_storage_alloc_gb,
        },
    };

    Ok(serde_json::to_value(dto)?)
}
pub fn aggregate_points(points: Vec<UniversalMetricPointDto>) -> Vec<UniversalMetricPointDto> {
    let mut map: HashMap<i64, Vec<UniversalMetricPointDto>> = HashMap::new();

    for point in points {
        map.entry(point.time.timestamp()).or_default().push(point);
    }

    let mut aggregated = Vec::new();

    for (ts, pts) in map {
        let len = pts.len() as f64;
        if len == 0.0 {
            continue;
        }

        let mut cpu_usage = 0.0;
        let mut memory_usage = 0.0;
        let mut fs_used = 0.0;
        let mut fs_capacity = 0.0;

        for p in &pts {
            cpu_usage += p.cpu_memory.cpu_usage_nano_cores.unwrap_or(0.0);
            memory_usage += p.cpu_memory.memory_usage_bytes.unwrap_or(0.0);

            if let Some(fs) = &p.filesystem {
                fs_used += fs.used_bytes.unwrap_or(0.0);
                fs_capacity += fs.capacity_bytes.unwrap_or(0.0);
            }
        }

        let time = chrono::DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now());

        aggregated.push(UniversalMetricPointDto {
            time,
            cpu_memory: CommonMetricValuesDto {
                cpu_usage_nano_cores: Some(cpu_usage / len),
                memory_usage_bytes: Some(memory_usage / len),
                ..Default::default()
            },
            filesystem: Some(FilesystemMetricDto {
                used_bytes: Some(fs_used / len),
                capacity_bytes: Some(fs_capacity / len),
                ..Default::default()
            }),
            ..Default::default()
        });
    }

    aggregated.sort_by_key(|p| p.time);
    aggregated
}

pub fn aggregate_cost_points(series: &[MetricSeriesDto]) -> Vec<UniversalMetricPointDto> {
    let mut map: HashMap<i64, (chrono::DateTime<Utc>, f64, f64, f64, f64)> = HashMap::new();

    for s in series {
        for point in &s.points {
            if let Some(cost) = &point.cost {
                let entry = map
                    .entry(point.time.timestamp())
                    .or_insert((point.time, 0.0, 0.0, 0.0, 0.0));

                entry.1 += cost.total_cost_usd.unwrap_or(0.0);
                entry.2 += cost.cpu_cost_usd.unwrap_or(0.0);
                entry.3 += cost.memory_cost_usd.unwrap_or(0.0);
                entry.4 += cost.storage_cost_usd.unwrap_or(0.0);
            }
        }
    }

    let mut aggregated = Vec::new();

    for (_, (time, total, cpu, mem, storage)) in map {
        aggregated.push(UniversalMetricPointDto {
            time,
            cost: Some(CostMetricDto {
                total_cost_usd: Some(total),
                cpu_cost_usd: Some(cpu),
                memory_cost_usd: Some(mem),
                storage_cost_usd: Some(storage),
            }),
            ..Default::default()
        });
    }

    aggregated.sort_by_key(|p| p.time);
    aggregated
}
