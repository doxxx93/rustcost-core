use crate::core::persistence::metrics::metric_fs_adapter_base_trait::MetricFsAdapterBase;
use crate::core::persistence::metrics::k8s::container::metric_container_entity::MetricContainerEntity;
use anyhow::{Result};
use chrono::{DateTime, NaiveDate, Utc};
use std::io::BufWriter;
use std::{
    fs::File,
    fs::{self, OpenOptions},
    io::Write,
    io::{BufRead, BufReader},
    path::Path,
};
use std::path::PathBuf;
use crate::core::persistence::metrics::k8s::path::{
    metric_k8s_container_key_minute_dir_path,
    metric_k8s_container_key_minute_file_path,
};

/// Adapter for container minute-level metrics.
/// Responsible for appending minute samples to the filesystem and cleaning up old data.
#[derive(Debug)]
pub struct MetricContainerMinuteFsAdapter;

impl MetricContainerMinuteFsAdapter {
    fn delete_batch(batch: &[PathBuf]) -> Result<()> {
        for path in batch {
            match fs::remove_file(path) {
                Ok(_) => tracing::info!("Deleted old container metric {:?}", path),
                Err(e) => tracing::error!("Failed to delete {:?}: {}", path, e),
            }
        }
        Ok(())
    }

    fn build_path_for(&self, container_key: &str, date: NaiveDate) -> PathBuf {
        let date_str = date.format("%Y-%m-%d").to_string();
        metric_k8s_container_key_minute_file_path(container_key, &date_str)
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

    // fn ensure_header(&self, path: &Path, file: &mut std::fs::File) -> Result<()> {
    //     if !path.exists() {
    //         let header = "TIME|CPU_USAGE_NANO_CORES|CPU_USAGE_CORE_NANO_SECONDS|MEMORY_USAGE_BYTES|MEMORY_WORKING_SET_BYTES|MEMORY_RSS_BYTES|MEMORY_PAGE_FAULTS|NETWORK_PHYSICAL_RX_BYTES|NETWORK_PHYSICAL_TX_BYTES|NETWORK_PHYSICAL_RX_ERRORS|NETWORK_PHYSICAL_TX_ERRORS|FS_USED_BYTES|FS_CAPACITY_BYTES|FS_ICONTAINERS_USED|FS_ICONTAINERS\n";
    //         file.write_all(header.as_bytes())?;
    //     }
    //     Ok(())
    // }

    fn opt(v: Option<u64>) -> String {
        v.map(|x| x.to_string()).unwrap_or_default()
    }
}

impl MetricFsAdapterBase<MetricContainerEntity> for MetricContainerMinuteFsAdapter {
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

    fn cleanup_old(&self, container_key: &str, before: DateTime<Utc>) -> Result<()> {
        const BATCH_SIZE: usize = 200;

        let dir = metric_k8s_container_key_minute_dir_path(container_key);
        if !dir.exists() {
            return Ok(());
        }

        let cutoff = before.date_naive();
        let mut batch: Vec<PathBuf> = Vec::with_capacity(BATCH_SIZE);

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            // Must be .rcd
            if path.extension().and_then(|e| e.to_str()) != Some("rcd") {
                continue;
            }

            // Extract valid UTF-8 filename stem
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.trim(),
                None => {
                    tracing::warn!("Skipping invalid UTF-8 file: {:?}", path);
                    continue;
                }
            };

            // Extract YYYY-MM-DD (first 10 chars)
            let date_str = &stem[..stem.len().min(10)];

            // Parse date
            let file_date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                Ok(d) => d,
                Err(e) => {
                    tracing::warn!("Skipping {:?}: invalid date '{}': {}", path, date_str, e);
                    continue;
                }
            };

            // Apply retention
            if file_date < cutoff {
                batch.push(path);

                // Flush batch
                if batch.len() >= BATCH_SIZE {
                    Self::delete_batch(&batch)?;
                    batch.clear();
                }
            }
        }

        // Flush leftovers
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
    ) -> Result<Vec<MetricContainerEntity>> {
        let mut all_rows = Vec::new();

        // 1️⃣ Iterate day-by-day across the requested range
        let mut current_date = start.date_naive();
        let end_date = end.date_naive();

        while current_date <= end_date {
            let path = self.build_path_for(object_name, current_date);
            let path_obj = Path::new(&path);

            if !path_obj.exists() {
                tracing::debug!("Minute metrics file missing for {} on {}", object_name, current_date);
                current_date = current_date.succ_opt().unwrap_or(current_date);
                continue;
            }

            // Safely open file
            let file = match File::open(&path_obj) {
                Ok(f) => f,
                Err(e) => {
                    tracing::warn!("Cannot open {:?}: {}", path_obj, e);
                    current_date = current_date.succ_opt().unwrap_or(current_date);
                    continue;
                }
            };

            let reader = BufReader::new(file);
            let mut lines = reader.lines();

            // Skip empty files
            let first_line = match lines.next() {
                Some(Ok(line)) => line,
                _ => {
                    tracing::debug!("Empty metric file for {} on {}", object_name, current_date);
                    current_date = current_date.succ_opt().unwrap_or(current_date);
                    continue;
                }
            };

            // 2️⃣ Handle header vs. data
            let header: Vec<&str>;
            let mut rows: Vec<MetricContainerEntity> = vec![];

            if first_line.starts_with("20") {
                // Treat as data (no header)
                header = vec![
                    "TIME", "CPU_USAGE_NANO_CORES", "CPU_USAGE_CORE_NANO_SECONDS",
                    "MEMORY_USAGE_BYTES", "MEMORY_WORKING_SET_BYTES", "MEMORY_RSS_BYTES",
                    "MEMORY_PAGE_FAULTS", "FS_USED_BYTES", "FS_CAPACITY_BYTES",
                    "FS_INODES_USED", "FS_INODES"
                ];

                if let Some(row) = Self::parse_line(&header, &first_line) {
                    if row.time >= start && row.time <= end {
                        rows.push(row);
                    }
                }
            } else {
                header = first_line.split('|').collect();
            }

            // 3️⃣ Process remaining lines safely
            for line_result in lines {
                let line = match line_result {
                    Ok(l) if !l.trim().is_empty() => l,
                    _ => continue,
                };

                if let Some(row) = Self::parse_line(&header, &line) {
                    if row.time < start {
                        continue;
                    }
                    if row.time > end {
                        break;
                    }
                    rows.push(row);
                } else {
                    tracing::warn!("Malformed line skipped in {:?}: {}", path_obj, line);
                }
            }

            all_rows.extend(rows);
            current_date = current_date.succ_opt().unwrap_or(current_date);
        }

        // 4️⃣ Sort and paginate
        all_rows.sort_by_key(|r| r.time);
        let start_idx = offset.unwrap_or(0);
        let limit = limit.unwrap_or(all_rows.len());
        let paginated = all_rows.into_iter().skip(start_idx).take(limit).collect::<Vec<_>>();

        tracing::debug!(
        "Returning {} rows for {} between {} and {}",
        paginated.len(),
        object_name,
        start,
        end
    );

        Ok(paginated)
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
}
