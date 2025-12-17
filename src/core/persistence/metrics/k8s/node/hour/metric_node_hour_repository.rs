use crate::core::persistence::metrics::k8s::node::hour::metric_node_hour_api_repository_trait::MetricNodeHourApiRepository;
use crate::core::persistence::metrics::k8s::node::hour::metric_node_hour_fs_adapter::MetricNodeHourFsAdapter;
use crate::core::persistence::metrics::k8s::node::hour::metric_node_hour_retention_repository_traits::MetricNodeHourRetentionRepository;
use crate::core::persistence::metrics::k8s::node::metric_node_entity::MetricNodeEntity;
use crate::core::persistence::metrics::metric_fs_adapter_base_trait::MetricFsAdapterBase;
use crate::domain::common::service::MetricRowRepository;
use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::error;

pub struct MetricNodeHourRepository {
    adapter: MetricNodeHourFsAdapter,
}

impl MetricNodeHourRepository {
    pub fn new() -> Self {
        Self {
            adapter: MetricNodeHourFsAdapter,
        }
    }
}

impl MetricRowRepository<MetricNodeEntity> for MetricNodeHourRepository {
    fn get_row_between(
        &self,
        object_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<MetricNodeEntity>> {
        MetricNodeHourApiRepository::get_row_between(self, object_name, start, end)
    }
}

impl Default for MetricNodeHourRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricNodeHourApiRepository for MetricNodeHourRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricNodeEntity> {
        &self.adapter
    }

    fn get_row_between(&self, node_key: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<MetricNodeEntity>> {
        self.adapter.get_row_between(start, end, node_key, None, None).map_err(|err| {
            error!(error = %err, node_key, "Failed to read node hour rows");
            err
        })
    }
}

impl MetricNodeHourRetentionRepository for MetricNodeHourRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricNodeEntity> {
        &self.adapter
    }

    fn cleanup_old(&self, node_name: &str, before: DateTime<Utc>) -> Result<()> {
        self.adapter.cleanup_old(node_name, before).map_err(|err| {
            error!(error = %err, node_name, "Failed to cleanup old node hour metrics");
            err
        })
    }
}
