use crate::core::persistence::metrics::k8s::node::day::metric_node_day_api_repository_trait::MetricNodeDayApiRepository;
use crate::core::persistence::metrics::k8s::node::day::metric_node_day_fs_adapter::MetricNodeDayFsAdapter;
use crate::core::persistence::metrics::k8s::node::day::metric_node_day_processor_repository_trait::MetricNodeDayProcessorRepository;
use crate::core::persistence::metrics::k8s::node::day::metric_node_day_retention_repository_traits::MetricNodeDayRetentionRepository;
use crate::core::persistence::metrics::k8s::node::metric_node_entity::MetricNodeEntity;
use crate::core::persistence::metrics::metric_fs_adapter_base_trait::MetricFsAdapterBase;
use crate::domain::common::service::MetricRowRepository;
use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::error;

pub struct MetricNodeDayRepository {
    adapter: MetricNodeDayFsAdapter,
}

impl MetricNodeDayRepository {
    pub fn new() -> Self {
        Self {
            adapter: MetricNodeDayFsAdapter,
        }
    }
}

impl MetricRowRepository<MetricNodeEntity> for MetricNodeDayRepository {
    fn get_row_between(
        &self,
        object_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<MetricNodeEntity>> {
        MetricNodeDayApiRepository::get_row_between(self, object_name, start, end)
    }
}

impl Default for MetricNodeDayRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricNodeDayApiRepository for MetricNodeDayRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricNodeEntity> {
        &self.adapter
    }

    fn get_row_between(&self, node_key: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<MetricNodeEntity>> {
        self.adapter.get_row_between(start, end, node_key, None, None).map_err(|err| {
            error!(error = %err, node_key, "Failed to read node day rows");
            err
        })
    }
}

impl MetricNodeDayRetentionRepository for MetricNodeDayRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricNodeEntity> {
        &self.adapter
    }

    fn cleanup_old(&self, node_name: &str, before: DateTime<Utc>) -> Result<()> {
        self.adapter.cleanup_old(node_name, before).map_err(|err| {
            error!(error = %err, node_name, "Failed to cleanup old node day metrics");
            err
        })
    }
}
impl MetricNodeDayProcessorRepository for MetricNodeDayRepository  {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricNodeEntity> {
        &self.adapter
    }

    fn append_row_aggregated(&self, node_uid: &str, start: DateTime<Utc>, end: DateTime<Utc>, now: DateTime<Utc>) -> anyhow::Result<()> {
        self.adapter.append_row_aggregated(node_uid, start, end, now)
    }
}