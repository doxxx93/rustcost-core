use crate::core::persistence::metrics::metric_fs_adapter_base_trait::MetricFsAdapterBase;
use crate::core::persistence::metrics::k8s::node::metric_node_entity::MetricNodeEntity;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use std::io::BufWriter;
use std::{
    fs::File,
    fs::{self, OpenOptions},
    io::Write,
    io::{BufRead, BufReader},
    path::Path,
};
use std::path::PathBuf;
use crate::core::persistence::metrics::k8s::node::hour::metric_node_hour_fs_adapter::MetricNodeHourFsAdapter;
use crate::core::persistence::metrics::k8s::path::{
    metric_k8s_node_key_day_dir_path,
    metric_k8s_node_key_day_file_path,
};

/// Adapter for node hour-level metrics.
/// Responsible for appending hour samples to the filesystem and cleaning up old data.
#[derive(Debug)]
pub struct MetricNodeDayFsAdapter;

impl MetricNodeDayFsAdapter {
    fn delete_batch(batch: &[PathBuf]) -> Result<()> {
        for path in batch {
            match fs::remove_file(path) {
                Ok(_) => tracing::info!("Deleted old metric file {:?}", path),
                Err(e) => tracing::error!("Failed to delete {:?}: {}", path, e),
            }
        }
        Ok(())
    }

    fn build_path_for(&self, node_key: &str, date: NaiveDate) -> PathBuf {
        let year_str = date.format("%Y").to_string();
        metric_k8s_node_key_day_file_path(node_key, &year_str)
    }

    fn parse_line(_header: &[&str], line: &str) -> Option<MetricNodeEntity> {
        use chrono::{DateTime, Utc};

        let parts: Vec<&str> = line.split('|').collect();
        if parts.is_empty() {
            return None;
        }

        let time = DateTime::parse_from_rfc3339(parts[0])
            .map(|dt| dt.with_timezone(&Utc))
            .ok()?;

        Some(MetricNodeEntity {
            time,
            cpu_usage_nano_cores: parts.get(1).and_then(|s| s.parse::<u64>().ok()),
            cpu_usage_core_nano_seconds: parts.get(2).and_then(|s| s.parse::<u64>().ok()),
            memory_usage_bytes: parts.get(3).and_then(|s| s.parse::<u64>().ok()),
            memory_working_set_bytes: parts.get(4).and_then(|s| s.parse::<u64>().ok()),
            memory_rss_bytes: parts.get(5).and_then(|s| s.parse::<u64>().ok()),
            memory_page_faults: parts.get(6).and_then(|s| s.parse::<u64>().ok()),
            network_physical_rx_bytes: parts.get(7).and_then(|s| s.parse::<u64>().ok()),
            network_physical_tx_bytes: parts.get(8).and_then(|s| s.parse::<u64>().ok()),
            network_physical_rx_errors: parts.get(9).and_then(|s| s.parse::<u64>().ok()),
            network_physical_tx_errors: parts.get(10).and_then(|s| s.parse::<u64>().ok()),
            fs_used_bytes: parts.get(11).and_then(|s| s.parse::<u64>().ok()),
            fs_capacity_bytes: parts.get(12).and_then(|s| s.parse::<u64>().ok()),
            fs_inodes_used: parts.get(13).and_then(|s| s.parse::<u64>().ok()),
            fs_inodes: parts.get(14).and_then(|s| s.parse::<u64>().ok()),
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

impl MetricFsAdapterBase<MetricNodeEntity> for MetricNodeDayFsAdapter {
    fn append_row(&self, node: &str, dto: &MetricNodeEntity, now: DateTime<Utc>) -> Result<()> {
        let now_date = now.date_naive();
        let path_str = self.build_path_for(node, now_date);
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
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
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
            Self::opt(dto.fs_used_bytes),
            Self::opt(dto.fs_capacity_bytes),
            Self::opt(dto.fs_inodes_used),
            Self::opt(dto.fs_inodes),
        );


        // ✅ write to buffer
        writer.write_all(row.as_bytes())?;

        // ✅ ensure everything flushed to disk
        writer.flush()?;
        Ok(())
    }

    /// Aggregate hour-level metrics into an dayly sample and append to day file.
    fn append_row_aggregated(
        &self,
        node_uid: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        now: DateTime<Utc>
    ) -> Result<()> {
        // --- 1️⃣ Load hour data
        let hour_adapter = MetricNodeHourFsAdapter;
        let rows = hour_adapter.get_row_between(start, end, node_uid, None, None)?;

        if rows.is_empty() {
            return Err(anyhow!("no hour data found for aggregation"));
        }

        // --- 2️⃣ Compute aggregates
        let first = rows.first().unwrap();
        let last = rows.last().unwrap();

        let avg = |f: fn(&MetricNodeEntity) -> Option<u64>| -> Option<u64> {
            let (sum, count): (u64, u64) =
                rows.iter().filter_map(f).fold((0, 0), |(s, c), v| (s + v, c + 1));
            if count > 0 {
                Some(sum / count)
            } else {
                None
            }
        };

        let delta = |f: fn(&MetricNodeEntity) -> Option<u64>| -> Option<u64> {
            match (f(first), f(last)) {
                (Some(a), Some(b)) if b >= a => Some(b - a),
                _ => None,
            }
        };

        let aggregated = MetricNodeEntity {
            time: end, // time marker = end of the aggregation window

            // CPU
            cpu_usage_nano_cores: avg(|r| r.cpu_usage_nano_cores),
            cpu_usage_core_nano_seconds: delta(|r| r.cpu_usage_core_nano_seconds),

            // Memory
            memory_usage_bytes: avg(|r| r.memory_usage_bytes),
            memory_working_set_bytes: avg(|r| r.memory_working_set_bytes),
            memory_rss_bytes: avg(|r| r.memory_rss_bytes),
            memory_page_faults: delta(|r| r.memory_page_faults),

            // Network
            network_physical_rx_bytes: delta(|r| r.network_physical_rx_bytes),
            network_physical_tx_bytes: delta(|r| r.network_physical_tx_bytes),
            network_physical_rx_errors: delta(|r| r.network_physical_rx_errors),
            network_physical_tx_errors: delta(|r| r.network_physical_tx_errors),

            // Filesystem
            fs_used_bytes: avg(|r| r.fs_used_bytes),
            fs_capacity_bytes: last.fs_capacity_bytes,
            fs_inodes_used: avg(|r| r.fs_inodes_used),
            fs_inodes: last.fs_inodes,
        };

        // --- 3️⃣ Append the aggregated row into the day-level file
        self.append_row(node_uid, &aggregated, now)?;

        Ok(())
    }


    fn cleanup_old(&self, node_uid: &str, before: DateTime<Utc>) -> Result<()> {
        const BATCH_SIZE: usize = 200;

        let dir = metric_k8s_node_key_day_dir_path(node_uid);
        if !dir.exists() {
            return Ok(());
        }

        let cutoff_year = before.year();
        let mut batch = Vec::with_capacity(BATCH_SIZE);

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            // Must be .rcd
            if path.extension().and_then(|e| e.to_str()) != Some("rcd") {
                continue;
            }

            // Extract filename stem as UTF-8
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.trim(),
                None => {
                    tracing::warn!("Skipping file with invalid UTF-8 name: {:?}", path);
                    continue;
                }
            };

            // Parse year (e.g., "2023")
            let file_year: i32 = match stem.parse() {
                Ok(y) => y,
                Err(_) => {
                    tracing::warn!("Skipping invalid year filename '{}'", stem);
                    continue;
                }
            };

            // Check retention policy
            if file_year < cutoff_year {
                batch.push(path);

                if batch.len() >= BATCH_SIZE {
                    Self::delete_batch(&batch)?;
                    batch.clear();
                }
            }
        }

        // Flush remaining batch
        if !batch.is_empty() {
            Self::delete_batch(&batch)?;
        }

        Ok(())
    }


    fn get_column_between(
        &self,
        column_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        object_name: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<MetricNodeEntity>> {
        let rows = self.get_row_between(start, end, object_name, limit, offset)?;
        let filtered: Vec<MetricNodeEntity> = rows
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
                        row.fs_used_bytes = None;
                        row.fs_capacity_bytes = None;
                        row.fs_inodes_used = None;
                        row.fs_inodes = None;
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
                        row.fs_used_bytes = None;
                        row.fs_capacity_bytes = None;
                        row.fs_inodes_used = None;
                        row.fs_inodes = None;
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
                        row.fs_used_bytes = None;
                        row.fs_capacity_bytes = None;
                        row.fs_inodes_used = None;
                        row.fs_inodes = None;
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
                        row.fs_used_bytes = None;
                        row.fs_capacity_bytes = None;
                        row.fs_inodes_used = None;
                        row.fs_inodes = None;
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
                        row.fs_used_bytes = None;
                        row.fs_capacity_bytes = None;
                        row.fs_inodes_used = None;
                        row.fs_inodes = None;
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
                        row.fs_used_bytes = None;
                        row.fs_capacity_bytes = None;
                        row.fs_inodes_used = None;
                        row.fs_inodes = None;
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
                        row.fs_used_bytes = None;
                        row.fs_capacity_bytes = None;
                        row.fs_inodes_used = None;
                        row.fs_inodes = None;
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
                        row.fs_used_bytes = None;
                        row.fs_capacity_bytes = None;
                        row.fs_inodes_used = None;
                        row.fs_inodes = None;
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
                        row.fs_used_bytes = None;
                        row.fs_capacity_bytes = None;
                        row.fs_inodes_used = None;
                        row.fs_inodes = None;
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
                        row.fs_used_bytes = None;
                        row.fs_capacity_bytes = None;
                        row.fs_inodes_used = None;
                        row.fs_inodes = None;
                        row.network_physical_tx_errors = keep;
                    }
                    "FS_USED_BYTES" => {
                        let keep = row.fs_used_bytes;
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
                        row.fs_capacity_bytes = None;
                        row.fs_inodes_used = None;
                        row.fs_inodes = None;
                        row.fs_used_bytes = keep;
                    }
                    "FS_CAPACITY_BYTES" => {
                        let keep = row.fs_capacity_bytes;
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
                        row.fs_used_bytes = None;
                        row.fs_inodes_used = None;
                        row.fs_inodes = None;
                        row.fs_capacity_bytes = keep;
                    }
                    "FS_INODES_USED" => {
                        let keep = row.fs_inodes_used;
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
                        row.fs_used_bytes = None;
                        row.fs_capacity_bytes = None;
                        row.fs_inodes = None;
                        row.fs_inodes_used = keep;
                    }
                    "FS_INODES" => {
                        let keep = row.fs_inodes;
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
                        row.fs_used_bytes = None;
                        row.fs_capacity_bytes = None;
                        row.fs_inodes_used = None;
                        row.fs_inodes = keep;
                    }
                    _ => {}
                }
                row
            })
            .collect();

        Ok(filtered)
    }

    fn get_row_between(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        object_name: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<MetricNodeEntity>> {

        // Collected result rows
        let mut data: Vec<MetricNodeEntity> = Vec::new();

        // 1️⃣ Determine year range from the requested time window
        // Files are stored per YEAR, so iteration must also be per YEAR
        let start_year = start.year();
        let end_year = end.year();

        // 2️⃣ Hard safety checks to prevent invalid or runaway queries
        if end_year < start_year {
            // Empty or invalid range
            return Ok(vec![]);
        }

        // Absolute safety fuse: prevent absurdly large scans
        if (end_year - start_year) > 10_000 {
            return Err(anyhow!("year range too large"));
        }

        // 3️⃣ Iterate year-by-year (NOT day-by-day)
        // Each yearly file is opened at most once
        for year in start_year..=end_year {
            let path = metric_k8s_node_key_day_file_path(object_name, &year.to_string());
            let path_obj = Path::new(&path);

            // Skip years with no data file
            if !path_obj.exists() {
                tracing::debug!(
                "Metric year file not found for {} in {}",
                object_name,
                year
            );
                continue;
            }

            // Open the yearly metric file
            let file = match File::open(&path_obj) {
                Ok(f) => f,
                Err(e) => {
                    tracing::warn!("Could not open {:?}: {}", path_obj, e);
                    continue;
                }
            };

            let reader = BufReader::new(file);

            // 4️⃣ Read file line-by-line
            // Assumption: rows are written in chronological order
            for line in reader.lines().flatten() {
                // Parse a single metric row
                let Some(row) = Self::parse_line(&[], &line) else {
                    continue;
                };

                // Skip rows before the requested start time
                if row.time < start {
                    continue;
                }

                // Stop reading this file once we exceed the end time
                // This is critical for performance
                if row.time > end {
                    break;
                }

                // Row is within [start, end] → collect it
                data.push(row);
            }
        }

        // 5️⃣ Final cleanup: sort and remove duplicates (defensive)
        data.sort_by_key(|r| r.time);
        data.dedup_by_key(|r| r.time);

        // 6️⃣ Apply pagination (offset + limit)
        let start_idx = offset.unwrap_or(0);
        let limit = limit.unwrap_or(data.len());

        Ok(
            data.into_iter()
                .skip(start_idx)
                .take(limit)
                .collect()
        )
    }


}
