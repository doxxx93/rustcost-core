use std::fs;
use std::path::PathBuf;

use anyhow::{ Result};
use chrono::{DateTime, Utc};

use crate::core::persistence::metrics::k8s::container::hour::{
    metric_container_hour_fs_adapter::MetricContainerHourFsAdapter,
    metric_container_hour_processor_repository_trait::MetricContainerHourProcessorRepository,
};
use crate::scheduler::tasks::processors::hour::container::metric_container_hour_processor_repository::MetricContainerHourProcessorRepositoryImpl;
use tracing::{debug};
use crate::core::persistence::metrics::k8s::path::metric_k8s_container_dir_path;
use crate::scheduler::tasks::utils::time_util::TimeUtils;

/// Aggregates all containers’ minute-level metrics into hour metrics.
///
/// This scans `data/metric/container/{container_key}/` and calls `append_row_aggregated()`
/// for each container directory, generating an hour summary.
pub async fn process_container_minute_to_hour(now: DateTime<Utc>) -> Result<()> {
    let (start, end) = TimeUtils::previous_hour_window(now)?;
    let base_dir = metric_k8s_container_dir_path();
    if !base_dir.exists() {
        debug!("No containers directory found at {:?}", base_dir);
        return Ok(());
    }

    let container_keys = collect_container_keys(&base_dir)?;
    if container_keys.is_empty() {
        debug!("No container metric directories found under {:?}", base_dir);
        return Ok(());
    }

    let repo = MetricContainerHourProcessorRepositoryImpl {
        adapter: MetricContainerHourFsAdapter,
    };

    process_all_containers(&repo, &container_keys, start, end, now);
    Ok(())
}

/// Collects all container UIDs (directory names) under the given base directory.
fn collect_container_keys(base_dir: &PathBuf) -> Result<Vec<String>> {
    let mut container_keys = Vec::new();

    for entry in fs::read_dir(base_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(container_key) = entry.file_name().to_str() {
                container_keys.push(container_key.to_string());
            }
        }
    }

    Ok(container_keys)
}

/// Aggregates minute-level data into hour data for all given containers.
fn process_all_containers<R: MetricContainerHourProcessorRepository>(
    repo: &R,
    container_keys: &[String],
    start: chrono::DateTime<Utc>,
    end: chrono::DateTime<Utc>,
    now: DateTime<Utc>
) {
    for container_key in container_keys {
        match repo.append_row_aggregated(container_key, start, end, now) {
            Ok(_) => debug!(
                "✅ Aggregated container '{}' minute metrics from {} → {}",
                container_key, start, end
            ),
            Err(err) => debug!(
                // TODO deleted container handling
                "⚠️ Failed to aggregate container '{}' metrics: {}",
                container_key, err
            ),
        }
    }
}
