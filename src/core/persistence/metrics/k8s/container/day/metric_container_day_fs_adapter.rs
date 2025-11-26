use crate::core::persistence::metrics::metric_fs_adapter_base_trait::MetricFsAdapterBase;
use crate::core::persistence::metrics::k8s::container::metric_container_entity::MetricContainerEntity;
use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, Utc, Datelike};
use std::io::BufWriter;
use std::{
    fs::File,
    fs::{self, OpenOptions},
    io::Write,
    io::{BufRead, BufReader},
    path::Path,
};
use std::path::PathBuf;
use crate::core::persistence::metrics::k8s::container::hour::metric_container_hour_fs_adapter::MetricContainerHourFsAdapter;
use crate::core::persistence::metrics::k8s::path::{metric_k8s_container_key_day_dir_path, metric_k8s_container_key_day_file_path};

/// Adapter for container hour-level metrics.
/// Responsible for appending hour samples to the filesystem and cleaning up old data.
#[derive(Debug)]
pub struct MetricContainerDayFsAdapter;

impl MetricContainerDayFsAdapter {
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
        metric_k8s_container_key_day_file_path(node_key, &year_str)
    }

    fn parse_line(header: &[&str], line: &str) -> Option<MetricContainerEntity> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() != header.len() {
            return None;
        }

        // TIME|CPU_USAGE_NANO_CORES|CPU_USAGE_CORE_NANO_SECONDS|... etc.
        let time = parts[0].parse::<DateTime<Utc>>().ok()?;
        Some(MetricContainerEntity {
            time,
            cpu_usage_nano_cores: parts[1].parse().ok(),
            cpu_usage_core_nano_seconds: parts[2].parse().ok(),
            memory_usage_bytes: parts[3].parse().ok(),
            memory_working_set_bytes: parts[4].parse().ok(),
            memory_rss_bytes: parts[5].parse().ok(),
            memory_page_faults: parts[6].parse().ok(),
            fs_used_bytes: parts[7].parse().ok(),
            fs_capacity_bytes: parts[8].parse().ok(),
            fs_inodes_used: parts[9].parse().ok(),
            fs_inodes: parts[10].parse().ok(),
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

impl MetricFsAdapterBase<MetricContainerEntity> for MetricContainerDayFsAdapter {
    fn append_row(&self, container: &str, dto: &MetricContainerEntity, now: DateTime<Utc>) -> Result<()> {
        let now_date = now.date_naive();
        let path_str = self.build_path_for(container, now_date);
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
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
            dto.time.to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
            Self::opt(dto.cpu_usage_nano_cores),
            Self::opt(dto.cpu_usage_core_nano_seconds),
            Self::opt(dto.memory_usage_bytes),
            Self::opt(dto.memory_working_set_bytes),
            Self::opt(dto.memory_rss_bytes),
            Self::opt(dto.memory_page_faults),
            // --- FS fields (rootfs + logs) ---
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
        container_uid: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        now: DateTime<Utc>
    ) -> Result<()> {
        let hour_adapter = MetricContainerHourFsAdapter;
        let rows = hour_adapter.get_row_between(start, end, container_uid, None, None)?;

        if rows.is_empty() {
            return Err(anyhow!("no hour data found for aggregation"));
        }

        // ---- 1️⃣ one-pass aggregation (FAST)
        let mut count = 0_u64;

        // sums
        let mut cpu_usage_sum = 0_u64;
        let mut mem_usage_sum = 0_u64;
        let mut mem_ws_sum = 0_u64;
        let mut mem_rss_sum = 0_u64;
        let mut fs_used_sum = 0_u64;
        let mut fs_inodes_used_sum = 0_u64;

        // first & last for delta tracking
        let first = &rows[0];
        let last  = rows.last().unwrap();

        for r in &rows {
            // avg fields
            if let Some(v) = r.cpu_usage_nano_cores       { cpu_usage_sum += v; }
            if let Some(v) = r.memory_usage_bytes         { mem_usage_sum += v; }
            if let Some(v) = r.memory_working_set_bytes   { mem_ws_sum += v; }
            if let Some(v) = r.memory_rss_bytes           { mem_rss_sum += v; }
            if let Some(v) = r.fs_used_bytes              { fs_used_sum += v; }
            if let Some(v) = r.fs_inodes_used             { fs_inodes_used_sum += v; }
            count += 1;
        }

        let avg_or_none = |sum: u64| -> Option<u64> {
            if count > 0 { Some(sum / count) } else { None }
        };

        // ---- 2️⃣ deltas (with counter reset detection)
        let delta = |f: fn(&MetricContainerEntity) -> Option<u64>| -> Option<u64> {
            match (f(first), f(last)) {
                (Some(a), Some(b)) => {
                    // if counter reset → treat as new counter
                    if b >= a { Some(b - a) } else { Some(b) }
                }
                _ => None,
            }
        };

        // ---- 3️⃣ final aggregated entity
        let aggregated = MetricContainerEntity {
            time: end,

            cpu_usage_nano_cores:            avg_or_none(cpu_usage_sum),
            cpu_usage_core_nano_seconds:     delta(|r| r.cpu_usage_core_nano_seconds),

            memory_usage_bytes:              avg_or_none(mem_usage_sum),
            memory_working_set_bytes:        avg_or_none(mem_ws_sum),
            memory_rss_bytes:                avg_or_none(mem_rss_sum),
            memory_page_faults:              delta(|r| r.memory_page_faults),

            fs_used_bytes:                   avg_or_none(fs_used_sum),
            fs_capacity_bytes:               last.fs_capacity_bytes,
            fs_inodes_used:                  avg_or_none(fs_inodes_used_sum),
            fs_inodes:                       last.fs_inodes,
        };

        // ---- 4️⃣ append row into correct day file
        self.append_row(container_uid, &aggregated, now)?;

        Ok(())
    }


    fn cleanup_old(&self, container_key: &str, before: DateTime<Utc>) -> Result<()> {
        const BATCH_SIZE: usize = 200;

        let dir = metric_k8s_container_key_day_dir_path(container_key);
        if !dir.exists() {
            return Ok(());
        }

        let cutoff_year = before.year();
        let mut batch = Vec::with_capacity(BATCH_SIZE);

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only delete *.rcd
            if path.extension().and_then(|e| e.to_str()) != Some("rcd") {
                continue;
            }

            // Extract filename stem safely
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.trim(),
                None => {
                    tracing::warn!("Skipping invalid UTF-8 filename: {:?}", path);
                    continue;
                }
            };

            // Expect: "2025.rcd"
            let file_year: i32 = match stem.parse() {
                Ok(y) => y,
                Err(_) => {
                    tracing::warn!("Skipping unknown filename '{}'", stem);
                    continue;
                }
            };

            // Check retention
            if file_year < cutoff_year {
                batch.push(path);

                if batch.len() >= BATCH_SIZE {
                    Self::delete_batch(&batch)?;
                    batch.clear();
                }
            }
        }

        // Flush remaining items
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
    ) -> Result<Vec<MetricContainerEntity>> {
        let rows = self.get_row_between(start, end, object_name, limit, offset)?;
        let filtered: Vec<MetricContainerEntity> = rows
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
                        row.fs_used_bytes = None;
                        row.fs_capacity_bytes = None;
                        row.fs_inodes_used = None;
                        row.fs_inodes = None;
                        row.memory_page_faults = keep;
                    }
                    "FS_USED_BYTES" => {
                        let keep = row.fs_used_bytes;
                        row.cpu_usage_nano_cores = None;
                        row.cpu_usage_core_nano_seconds = None;
                        row.memory_usage_bytes = None;
                        row.memory_working_set_bytes = None;
                        row.memory_rss_bytes = None;
                        row.memory_page_faults = None;
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
    ) -> Result<Vec<MetricContainerEntity>> {
        const HEADER: [&str; 11] = [
            "TIME",
            "CPU_USAGE_NANO_CORES",
            "CPU_USAGE_CORE_NANO_SECONDS",
            "MEMORY_USAGE_BYTES",
            "MEMORY_WORKING_SET_BYTES",
            "MEMORY_RSS_BYTES",
            "MEMORY_PAGE_FAULTS",
            "FS_USED_BYTES",
            "FS_CAPACITY_BYTES",
            "FS_INODES_USED",
            "FS_INODES",
        ];

        let mut data = Vec::new();
        let mut current_date = start.naive_utc().date();
        let end_date = end.naive_utc().date();

        // ✅ Iterate over each *year* that overlaps the range
        while current_date.year() <= end_date.year() {
            let path = self.build_path_for(object_name, current_date);
            let path_obj = Path::new(&path);

            if !path_obj.exists() {
                current_date = NaiveDate::from_ymd_opt(current_date.year() + 1, 1, 1)
                    .unwrap_or(current_date);
                continue;
            }

            if let Ok(file) = File::open(&path_obj) {
                let reader = BufReader::new(file);
                for line_result in reader.lines() {
                    let line = match line_result {
                        Ok(ref l) if !l.trim().is_empty() => l,
                        _ => continue,
                    };
                    if let Some(row) = Self::parse_line(&HEADER, line) {
                        if row.time < start {
                            continue;
                        }
                        if row.time > end {
                            break;
                        }
                        data.push(row);
                    }
                }
            }

            // move to next year
            current_date = NaiveDate::from_ymd_opt(current_date.year() + 1, 1, 1)
                .unwrap_or(current_date);
        }

        // ✅ Sort and paginate
        data.sort_by_key(|r| r.time);
        let start_idx = offset.unwrap_or(0);
        let limit = limit.unwrap_or(data.len());
        let paginated: Vec<_> = data.into_iter().skip(start_idx).take(limit).collect();

        Ok(paginated)
    }

}
