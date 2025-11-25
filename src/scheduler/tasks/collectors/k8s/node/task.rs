use chrono::{DateTime, Utc};
use crate::core::persistence::info::k8s::node::info_node_collector_repository_trait::InfoNodeCollectorRepository;
use crate::core::persistence::metrics::k8s::node::minute::metric_node_minute_collector_repository_trait::MetricNodeMinuteCollectorRepository;
use crate::core::persistence::metrics::k8s::node::minute::metric_node_minute_fs_adapter::MetricNodeMinuteFsAdapter;
use crate::scheduler::tasks::collectors::k8s::node::info_node_minute_collector_repository::InfoNodeCollectorRepositoryImpl;
use crate::scheduler::tasks::collectors::k8s::node::mappers::{map_summary_to_metrics, map_summary_to_node_info};
use crate::core::client::mappers::map_node_to_info_entity;
use crate::scheduler::tasks::collectors::k8s::node::metric_node_minute_collector_repository::MetricNodeMinuteCollectorRepositoryImpl;
use crate::core::client::kube_resources::Node;
use crate::scheduler::tasks::collectors::k8s::summary_dto::Summary;

pub async fn handle_node(summary: &Summary, now: DateTime<Utc>) -> Result<bool, anyhow::Error> {
    let node_name = &summary.node.node_name;

    // Step 1: Write info.rci if missing
    let info_repo = InfoNodeCollectorRepositoryImpl::default();
    let node_info = map_summary_to_node_info(summary, now);
    let created = info_repo.create_if_missing(node_name, &node_info)?;

    // Step 2: Append metrics
    let metrics_dto = map_summary_to_metrics(summary, now);
    let metric_repo = MetricNodeMinuteCollectorRepositoryImpl {
        adapter: MetricNodeMinuteFsAdapter,
    };
    metric_repo.append_row(node_name, &metrics_dto)?; // ✅ correct method

    Ok(created)
}

/// Checks cluster nodes and updates node info files if any node is new or changed.
/// Updates local node info for nodes whose names appear in `updated_nodes`.
///
/// - Reads data from the `NodeList` (fetched from K8s API)
/// - Updates only nodes present in `updated_nodes`
/// - Returns the updated `NodeList` for potential reuse
pub async fn update_node_info(
    node: Node,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {

    let repo = InfoNodeCollectorRepositoryImpl::default();

    let node_info = map_node_to_info_entity(&node)?;

    repo.update(&node_info)
        .expect("Failed to update node info in InfoNodeCollectorRepository");

    Ok(())
}
