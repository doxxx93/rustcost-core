use std::fs;
use std::path::{PathBuf};

use anyhow::{Result};
use chrono::{DateTime,  Utc};

use crate::core::persistence::metrics::k8s::node::day::{
    metric_node_day_fs_adapter::MetricNodeDayFsAdapter,
    metric_node_day_processor_repository_trait::MetricNodeDayProcessorRepository,
};
use tracing::{debug, error};
use crate::core::persistence::metrics::k8s::path::metric_k8s_node_dir_path;
use crate::scheduler::tasks::processors::day::node::metric_node_day_processor_repository::MetricNodeDayProcessorRepositoryImpl;
use crate::scheduler::tasks::utils::time_util::TimeUtils;

/// Aggregates all nodes’ minute-level metrics into dayly metrics.
///
/// This scans `data/metric/node/{node_name}/` and calls `append_row_aggregated()`
/// for each node directory, generating an dayly summary.
pub async fn process_node_hour_to_day(now: DateTime<Utc>) -> Result<()> {
    let (start, end) = TimeUtils::previous_day_window(now);
    let base_dir = metric_k8s_node_dir_path();

    if !base_dir.exists() {
        debug!("No nodes directory found at {:?}", base_dir);
        return Ok(());
    }

    let node_names = collect_node_names(&base_dir)?;
    if node_names.is_empty() {
        debug!("No node metric directories found under {:?}", base_dir);
        return Ok(());
    }

    let repo = MetricNodeDayProcessorRepositoryImpl {
        adapter: MetricNodeDayFsAdapter,
    };

    process_all_nodes(&repo, &node_names, start, end, now);
    Ok(())
}

/// Collects all node UIDs (directory names) under the given base directory.
fn collect_node_names(base_dir: &PathBuf) -> Result<Vec<String>> {
    let mut node_names = Vec::new();

    for entry in fs::read_dir(base_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(node_name) = entry.file_name().to_str() {
                node_names.push(node_name.to_string());
            }
        }
    }

    Ok(node_names)
}

/// Aggregates minute-level data into dayly data for all given nodes.
fn process_all_nodes<R: MetricNodeDayProcessorRepository>(
    repo: &R,
    node_names: &[String],
    start: chrono::DateTime<Utc>,
    end: chrono::DateTime<Utc>,
    now: DateTime<Utc>
) {
    for node_name in node_names {
        match repo.append_row_aggregated(node_name, start, end, now) {
            Ok(_) => debug!(
                "✅ Aggregated node '{}' minute metrics from {} → {}",
                node_name, start, end
            ),
            Err(err) => error!(
                "⚠️ Failed to aggregate node '{}' metrics: {}",
                node_name, err
            ),
        }
    }
}
