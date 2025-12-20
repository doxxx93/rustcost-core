use crate::core::persistence::metrics::metric_fs_adapter_base_trait::MetricFsAdapterBase;
use crate::core::persistence::metrics::k8s::pod::metric_pod_entity::MetricPodEntity;
use anyhow::{anyhow,  Result};
use chrono::{DateTime, NaiveDate, Datelike, Utc};
use std::io::BufWriter;
use std::{
    fs::File,
    fs::{self, OpenOptions},
    io::Write,
    io::{BufRead, BufReader},
    path::Path,
};
use std::path::PathBuf;
use crate::core::persistence::metrics::k8s::pod::minute::metric_pod_minute_fs_adapter::MetricPodMinuteFsAdapter;
use crate::core::persistence::metrics::k8s::path::{
    metric_k8s_pod_key_hour_dir_path,
    metric_k8s_pod_key_hour_file_path,
};

/// Adapter for pod minute-level metrics.
/// Responsible for appending minute samples to the filesystem and cleaning up old data.
#[derive(Debug)]
pub struct MetricPodHourFsAdapter;

impl MetricPodHourFsAdapter {

    /// Delete a batch of files safely
    fn delete_batch(batch: &[PathBuf]) -> Result<()> {
        for path in batch {
            if let Err(e) = fs::remove_file(path) {
                // Continue deleting others — best-effort cleanup
                tracing::error!("Failed to delete {:?}: {}", path, e);
            } else {
                tracing::info!("Deleted old metric file {:?}", path);
            }
        }
        Ok(())
    }
    fn build_path_for(&self, pod_uid: &str, date: NaiveDate) -> PathBuf {
        let month_str = date.format("%Y-%m").to_string();
        metric_k8s_pod_key_hour_file_path(pod_uid, &month_str)
    }

    fn parse_line(header: &[&str], line: &str) -> Option<MetricPodEntity> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() != header.len() {
            return None;
        }

        // TIME|CPU_USAGE_NANO_CORES|CPU_USAGE_CORE_NANO_SECONDS|... etc.
        let time = parts[0].parse::<DateTime<Utc>>().ok()?;
        Some(MetricPodEntity {
            time,
            cpu_usage_nano_cores: parts[1].parse().ok(),
            cpu_usage_core_nano_seconds: parts[2].parse().ok(),
            memory_usage_bytes: parts[3].parse().ok(),
            memory_working_set_bytes: parts[4].parse().ok(),
            memory_rss_bytes: parts[5].parse().ok(),
            memory_page_faults: parts[6].parse().ok(),
            network_physical_rx_bytes: parts[7].parse().ok(),
            network_physical_tx_bytes: parts[8].parse().ok(),
            network_physical_rx_errors: parts[9].parse().ok(),
            network_physical_tx_errors: parts[10].parse().ok(),
            es_used_bytes: parts[11].parse().ok(),
            es_capacity_bytes: parts[12].parse().ok(),
            es_inodes_used: parts[13].parse().ok(),
            es_inodes: parts[14].parse().ok(),
            pv_used_bytes: parts[15].parse().ok(),
            pv_capacity_bytes: parts[16].parse().ok(),
            pv_inodes_used: parts[17].parse().ok(),
            pv_inodes: parts[18].parse().ok(),
        })
    }

    // fn ensure_header(file: &mut File) -> Result<()> {
    //     if file.metadata()?.len() == 0 {
    //         let header = "TIME|CPU_USAGE_NANO_CORES|CPU_USAGE_CORE_NANO_SECONDS|MEMORY_USAGE_BYTES|MEMORY_WORKING_SET_BYTES|MEMORY_RSS_BYTES|MEMORY_PAGE_FAULTS|NETWORK_PHYSICAL_RX_BYTES|NETWORK_PHYSICAL_TX_BYTES|NETWORK_PHYSICAL_RX_ERRORS|NETWORK_PHYSICAL_TX_ERRORS|ES_USED_BYTES|ES_CAPACITY_BYTES|ES_INODES_USED|ES_INODES|PV_USED_BYTES|PV_CAPACITY_BYTES|PV_INODES_USED|PV_INODES\n";
    //         file.write_all(header.as_bytes())?;
    //     }
    //     Ok(())
    // }


    fn opt(v: Option<u64>) -> String {
        v.map(|x| x.to_string()).unwrap_or_default()
    }
}

impl MetricFsAdapterBase<MetricPodEntity> for MetricPodHourFsAdapter {
    fn append_row(&self, pod: &str, dto: &MetricPodEntity, _now: DateTime<Utc>) -> Result<()> {
        // IMPORTANT: partition by the metric timestamp, not "now".
        // This prevents late aggregation/backfill data from being written into the wrong file.
        let dto_date = dto.time.date_naive();
        let path_str = self.build_path_for(pod, dto_date);
        let path = Path::new(&path_str);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // let new = !path.exists();

        // ✅ open file and wrap in BufWriter
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        let mut writer = BufWriter::new(file);

        // Write header if file newly created
        // if new {
        //     self.ensure_header(path, &mut writer)?;
        // }

        // Format the row
        let row = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
            dto.time.to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
            Self::opt(dto.cpu_usage_nano_cores),
            Self::opt(dto.cpu_usage_core_nano_seconds),
            Self::opt(dto.memory_usage_bytes),
            Self::opt(dto.memory_working_set_bytes),
            Self::opt(dto.memory_rss_bytes),
            Self::opt(dto.memory_page_faults),
            Self::opt(dto.network_physical_rx_bytes),
            Self::opt(dto.network_physical_tx_bytes),
            Self::opt(dto.network_physical_rx_errors),
            Self::opt(dto.network_physical_tx_errors),
            Self::opt(dto.es_used_bytes),
            Self::opt(dto.es_capacity_bytes),
            Self::opt(dto.es_inodes_used),
            Self::opt(dto.es_inodes),
            Self::opt(dto.pv_used_bytes),
            Self::opt(dto.pv_capacity_bytes),
            Self::opt(dto.pv_inodes_used),
            Self::opt(dto.pv_inodes),
        );


        // ✅ write to buffer
        writer.write_all(row.as_bytes())?;

        // ✅ ensure everything flushed to disk
        writer.flush()?;
        Ok(())
    }

    /// Aggregate minute-level metrics into an hour sample and append to hour file.
    fn append_row_aggregated(
        &self,
        pod_uid: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        now: DateTime<Utc>
    ) -> Result<()> {
        // 1) Load minute-level samples in [start, end].
        let minute_adapter = MetricPodMinuteFsAdapter;
        let mut rows = minute_adapter.get_row_between(start, end, pod_uid, None, None)?;

        if rows.is_empty() {
            return Err(anyhow!("no minute data found for aggregation"));
        }

        // Ensure chronological order for weighted averaging and counter increase summation.
        rows.sort_by_key(|r| r.time);

        let first = rows.first().unwrap();
        let last = rows.last().unwrap();

        // --- Time-weighted average for gauge metrics (state values).
        // We assume each sample holds its value until the next sample timestamp.
        // This is robust to missing samples and irregular sampling intervals.
        let twa_u64 = |f: fn(&MetricPodEntity) -> Option<u64>| -> Option<u64> {
            let mut pts: Vec<(DateTime<Utc>, u64)> =
                rows.iter().filter_map(|r| f(r).map(|v| (r.time, v))).collect();

            if pts.is_empty() {
                return None;
            }
            pts.sort_by_key(|(t, _)| *t);

            let window_ns = (end - start).num_nanoseconds()? as f64;
            if window_ns <= 0.0 {
                // Degenerate window: return the last known value as a safe fallback.
                return Some(pts.last().unwrap().1);
            }

            let mut area: f64 = 0.0;

            for i in 0..pts.len() {
                let (t_i, v_i) = pts[i];
                let seg_end = if i + 1 < pts.len() { pts[i + 1].0 } else { end };

                // Clamp segment boundaries to [start, end].
                let seg_start = std::cmp::max(t_i, start);
                let seg_end = std::cmp::min(seg_end, end);

                if seg_end > seg_start {
                    let seg_ns = (seg_end - seg_start).num_nanoseconds()? as f64;
                    area += (v_i as f64) * seg_ns;
                }
            }

            Some((area / window_ns).round() as u64)
        };

        // --- Reset-aware sum of increases for counter metrics.
        // Normal case: add positive deltas (cur - prev).
        // Reset case: if cur < prev, assume counter restarted at 0; add cur (Prometheus increase-like).
        let sum_increase_reset_aware = |f: fn(&MetricPodEntity) -> Option<u64>| -> Option<u64> {
            let mut acc: u64 = 0;
            let mut prev: Option<u64> = None;
            let mut has_pair = false;

            for r in &rows {
                let cur = match f(r) {
                    Some(v) => v,
                    None => continue,
                };

                if let Some(p) = prev {
                    has_pair = true;
                    if cur >= p {
                        acc = acc.saturating_add(cur - p);
                    } else {
                        // Counter reset compensation.
                        acc = acc.saturating_add(cur);
                    }
                }
                prev = Some(cur);
            }

            if has_pair { Some(acc) } else { None }
        };

        // --- Supply/capacity snapshots: prefer max (conservative), fallback to last.
        let max_u64 = |f: fn(&MetricPodEntity) -> Option<u64>| -> Option<u64> {
            rows.iter().filter_map(f).max()
        };

        // 2) Build hour-level aggregated row (timestamp = end of window).
        let aggregated = MetricPodEntity {
            time: end,

            // CPU
            cpu_usage_nano_cores: twa_u64(|r| r.cpu_usage_nano_cores),
            cpu_usage_core_nano_seconds: sum_increase_reset_aware(|r| r.cpu_usage_core_nano_seconds),

            // Memory
            memory_usage_bytes: twa_u64(|r| r.memory_usage_bytes),
            memory_working_set_bytes: twa_u64(|r| r.memory_working_set_bytes),
            memory_rss_bytes: twa_u64(|r| r.memory_rss_bytes),
            memory_page_faults: sum_increase_reset_aware(|r| r.memory_page_faults),

            // Network
            network_physical_rx_bytes: sum_increase_reset_aware(|r| r.network_physical_rx_bytes),
            network_physical_tx_bytes: sum_increase_reset_aware(|r| r.network_physical_tx_bytes),
            network_physical_rx_errors: sum_increase_reset_aware(|r| r.network_physical_rx_errors),
            network_physical_tx_errors: sum_increase_reset_aware(|r| r.network_physical_tx_errors),

            // Ephemeral storage
            es_used_bytes: twa_u64(|r| r.es_used_bytes),
            es_capacity_bytes: max_u64(|r| r.es_capacity_bytes).or(last.es_capacity_bytes),
            es_inodes_used: twa_u64(|r| r.es_inodes_used),
            es_inodes: max_u64(|r| r.es_inodes).or(last.es_inodes),

            // Persistent storage
            pv_used_bytes: twa_u64(|r| r.pv_used_bytes),
            pv_capacity_bytes: max_u64(|r| r.pv_capacity_bytes).or(last.pv_capacity_bytes),
            pv_inodes_used: twa_u64(|r| r.pv_inodes_used),
            pv_inodes: max_u64(|r| r.pv_inodes).or(last.pv_inodes),
        };

        // 3) Append the aggregated sample (storage partitioning uses aggregated.time internally).
        self.append_row(pod_uid, &aggregated, now)?;

        Ok(())
    }


    fn cleanup_old(&self, pod_uid: &str, before: DateTime<Utc>) -> Result<()> {
        const BATCH_SIZE: usize = 200;
        let dir = metric_k8s_pod_key_hour_dir_path(pod_uid);
        if !dir.exists() {
            return Ok(());
        }

        // Normalize cutoff to YYYY-MM-01
        let before_month = NaiveDate::from_ymd_opt(before.year(), before.month(), 1)
            .ok_or_else(|| anyhow!("Invalid 'before' month {}-{}", before.year(), before.month()))?;

        let mut batch: Vec<PathBuf> = Vec::with_capacity(BATCH_SIZE);

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process *.rcd
            if path.extension().and_then(|e| e.to_str()) != Some("rcd") {
                continue;
            }

            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.trim(),
                None => {
                    tracing::warn!(
                    "Skipping file with non-UTF8 name in {:?}",
                    path
                );
                    continue;
                }
            };

            // Expect filename like "2025-02"
            let parts: Vec<&str> = stem.split('-').collect();
            if parts.len() != 2 {
                tracing::warn!("Skipping unexpected filename '{}'", stem);
                continue;
            }

            let year: i32 = match parts[0].parse() {
                Ok(v) => v,
                Err(_) => {
                    tracing::warn!("Invalid year '{}' in '{}'", parts[0], stem);
                    continue;
                }
            };

            let month: u32 = match parts[1].parse() {
                Ok(v) if (1..=12).contains(&v) => v,
                _ => {
                    tracing::warn!("Invalid month '{}' in '{}'", parts[1], stem);
                    continue;
                }
            };

            let file_month = match NaiveDate::from_ymd_opt(year, month, 1) {
                Some(d) => d,
                None => {
                    tracing::warn!("Invalid date '{}-{}' in '{}'", year, month, stem);
                    continue;
                }
            };

            // Compare normalized YYYY-MM-01
            if file_month < before_month {
                batch.push(path);

                // Batching to avoid blocking for too long
                if batch.len() >= BATCH_SIZE {
                    Self::delete_batch(&batch)?;
                    batch.clear();
                }
            }
        }

        // Delete remaining
        if !batch.is_empty() {
            Self::delete_batch(&batch)?;
        }

        Ok(())
    }


    fn get_row_between(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        object_name: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<MetricPodEntity>> {
        let mut data: Vec<MetricPodEntity> = vec![];

        // 1️⃣ Iterate month by month between start and end
        let mut current_date = NaiveDate::from_ymd_opt(start.year(), start.month() as u32, 1)
            .expect("valid start date");
        let end_date = NaiveDate::from_ymd_opt(end.year(), end.month() as u32, 1)
            .expect("valid end date");

        while current_date <= end_date {
            let path = self.build_path_for(object_name, current_date);
            let path_obj = Path::new(&path);

            if !path_obj.exists() {
                tracing::debug!(
                "Hour metrics file missing for {} at month {}",
                object_name,
                current_date.format("%Y-%m")
            );
                // Move to next month
                current_date = if current_date.month() == 12 {
                    NaiveDate::from_ymd_opt(current_date.year() + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(current_date.year(), current_date.month() + 1, 1).unwrap()
                };
                continue;
            }

            let file = match File::open(&path_obj) {
                Ok(f) => f,
                Err(e) => {
                    tracing::warn!("Could not open {:?}: {}", path_obj, e);
                    current_date = if current_date.month() == 12 {
                        NaiveDate::from_ymd_opt(current_date.year() + 1, 1, 1).unwrap()
                    } else {
                        NaiveDate::from_ymd_opt(current_date.year(), current_date.month() + 1, 1).unwrap()
                    };
                    continue;
                }
            };

            let reader = BufReader::new(file);
            let mut lines = reader.lines();

            // 2️⃣ Try to read the first line (header or data)
            let first_line_opt = lines.next();
            if first_line_opt.is_none() {
                current_date = if current_date.month() == 12 {
                    NaiveDate::from_ymd_opt(current_date.year() + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(current_date.year(), current_date.month() + 1, 1).unwrap()
                };
                continue;
            }

            let first_line = first_line_opt.unwrap_or_else(|| Ok(String::new()))?;
            let mut rows: Vec<MetricPodEntity> = vec![];
            let header: Vec<&str>;

            // Handle header or first data line
            if first_line.starts_with("20") {
                header = vec![
                    "TIME", "CPU_USAGE_NANO_CORES", "CPU_USAGE_CORE_NANO_SECONDS",
                    "MEMORY_USAGE_BYTES", "MEMORY_WORKING_SET_BYTES", "MEMORY_RSS_BYTES",
                    "MEMORY_PAGE_FAULTS", "NETWORK_PHYSICAL_RX_BYTES", "NETWORK_PHYSICAL_TX_BYTES",
                    "NETWORK_PHYSICAL_RX_ERRORS", "NETWORK_PHYSICAL_TX_ERRORS",
                    "ES_USED_BYTES", "ES_CAPACITY_BYTES", "ES_INODES_USED", "ES_INODES",
                    "PV_USED_BYTES", "PV_CAPACITY_BYTES", "PV_INODES_USED", "PV_INODES"
                ];

                if let Some(row) = Self::parse_line(&header, &first_line) {
                    if row.time >= start && row.time <= end {
                        rows.push(row);
                    }
                }
            } else {
                header = first_line.split('|').collect();
            }

            // 3️⃣ Process the rest of the lines
            for line in lines.flatten() {
                if let Some(row) = Self::parse_line(&header, &line) {
                    if row.time < start {
                        continue;
                    }
                    if row.time > end {
                        break;
                    }
                    rows.push(row);
                }
            }

            data.append(&mut rows);

            // Move to next month
            current_date = if current_date.month() == 12 {
                NaiveDate::from_ymd_opt(current_date.year() + 1, 1, 1).unwrap()
            } else {
                NaiveDate::from_ymd_opt(current_date.year(), current_date.month() + 1, 1).unwrap()
            };
        }

        // 4️⃣ Sort and paginate
        data.sort_by_key(|r| r.time);

        let start_idx = offset.unwrap_or(0);
        let limit = limit.unwrap_or(data.len());
        let slice: Vec<_> = data.into_iter().skip(start_idx).take(limit).collect();

        tracing::debug!(
        "Returning {} hour rows for {} between {} and {}",
        slice.len(),
        object_name,
        start,
        end
    );

        Ok(slice)
    }


    fn get_column_between(
        &self,
        column_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        object_name: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<MetricPodEntity>> {
        let rows = self.get_row_between(start, end, object_name, limit, offset)?;
        let filtered: Vec<MetricPodEntity> = rows
            .into_iter()
            .map(|mut row| {
                match column_name {
                    "CPU_USAGE_NANO_CORES" => {
                        let keep = row.cpu_usage_nano_cores;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.cpu_usage_nano_cores = keep;
                    }
                    "CPU_USAGE_CORE_NANO_SECONDS" => {
                        let keep = row.cpu_usage_core_nano_seconds;
                        row.cpu_usage_nano_cores = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.cpu_usage_core_nano_seconds = keep;
                    }
                    "MEMORY_USAGE_BYTES" => {
                        let keep = row.memory_usage_bytes;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.memory_usage_bytes = keep;
                    }
                    "MEMORY_WORKING_SET_BYTES" => {
                        let keep = row.memory_working_set_bytes;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.memory_working_set_bytes = keep;
                    }
                    "MEMORY_RSS_BYTES" => {
                        let keep = row.memory_rss_bytes;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.memory_rss_bytes = keep;
                    }
                    "MEMORY_PAGE_FAULTS" => {
                        let keep = row.memory_page_faults;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.memory_page_faults = keep;
                    }
                    "NETWORK_PHYSICAL_RX_BYTES" => {
                        let keep = row.network_physical_rx_bytes;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.network_physical_rx_bytes = keep;
                    }
                    "NETWORK_PHYSICAL_TX_BYTES" => {
                        let keep = row.network_physical_tx_bytes;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.network_physical_tx_bytes = keep;
                    }
                    "NETWORK_PHYSICAL_RX_ERRORS" => {
                        let keep = row.network_physical_rx_errors;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.network_physical_rx_errors = keep;
                    }
                    "NETWORK_PHYSICAL_TX_ERRORS" => {
                        let keep = row.network_physical_tx_errors;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.network_physical_tx_errors = keep;
                    }
                    "ES_USED_BYTES" => {
                        let keep = row.es_used_bytes;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.es_used_bytes = keep;
                    }
                    "ES_CAPACITY_BYTES" => {
                        let keep = row.es_capacity_bytes;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.es_capacity_bytes = keep;
                    }
                    "ES_INODES_USED" => {
                        let keep = row.es_inodes_used;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.es_inodes_used = keep;
                    }
                    "ES_INODES" => {
                        let keep = row.es_inodes;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.es_inodes = keep;
                    }
                    "PV_USED_BYTES" => {
                        let keep = row.pv_used_bytes;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.pv_used_bytes = keep;
                    }
                    "PV_CAPACITY_BYTES" => {
                        let keep = row.pv_capacity_bytes;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = None;
                        row.pv_capacity_bytes = keep;
                    }
                    "PV_INODES_USED" => {
                        let keep = row.pv_inodes_used;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes = None;
                        row.pv_inodes_used = keep;
                    }
                    "PV_INODES" => {
                        let keep = row.pv_inodes;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
                        row.network_physical_rx_bytes = None;
                        row.network_physical_tx_bytes = None;
                        row.network_physical_rx_errors = None;
                        row.network_physical_tx_errors = None;
                        row.es_used_bytes = None;
                        row.es_capacity_bytes = None;
                        row.es_inodes_used = None;
                        row.es_inodes = None;
                        row.pv_used_bytes = None;
                        row.pv_capacity_bytes = None;
                        row.pv_inodes_used = None;
                        row.pv_inodes = keep;
                    }
                    _ => {}
                }
                row
            })
            .collect();

        Ok(filtered)
    }
}
