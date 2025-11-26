use crate::core::persistence::metrics::k8s::node::metric_node_entity::MetricNodeEntity;
use crate::core::persistence::metrics::k8s::node::minute::metric_node_minute_api_repository_trait::MetricNodeMinuteApiRepository;
use crate::core::persistence::metrics::k8s::node::minute::metric_node_minute_collector_repository_trait::MetricNodeMinuteCollectorRepository;
use crate::core::persistence::metrics::k8s::node::minute::metric_node_minute_fs_adapter::MetricNodeMinuteFsAdapter;
use crate::core::persistence::metrics::k8s::node::minute::metric_node_minute_processor_repository_trait::MetricNodeMinuteProcessorRepository;
use crate::core::persistence::metrics::k8s::node::minute::metric_node_minute_retention_repository_traits::MetricNodeMinuteRetentionRepository;
use crate::core::persistence::metrics::metric_fs_adapter_base_trait::MetricFsAdapterBase;
use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::error;

pub struct MetricNodeMinuteRepository {
    adapter: MetricNodeMinuteFsAdapter,
}

impl MetricNodeMinuteRepository {
    pub fn new() -> Self {
        Self {
            adapter: MetricNodeMinuteFsAdapter,
        }
    }
}

impl Default for MetricNodeMinuteRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricNodeMinuteApiRepository for MetricNodeMinuteRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricNodeEntity> {
        &self.adapter
    }

    fn get_row_between(&self, node_key: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<MetricNodeEntity>> {
        self.adapter.get_row_between(start, end, node_key, None, None).map_err(|err| {
            error!(error = %err, node_key, "Failed to read node minute rows");
            err
        })
    }
}

impl MetricNodeMinuteCollectorRepository for MetricNodeMinuteRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricNodeEntity> {
        &self.adapter
    }

    fn append_row(&self, node_name: &str, data: &MetricNodeEntity, now: DateTime<Utc>) -> Result<()> {
        self.adapter.append_row(node_name, data, now).map_err(|err| {
            error!(error = %err, node_name, "Failed to append node minute row");
            err
        })
    }
}

impl MetricNodeMinuteProcessorRepository for MetricNodeMinuteRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricNodeEntity> {
        &self.adapter
    }
}

impl MetricNodeMinuteRetentionRepository for MetricNodeMinuteRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricNodeEntity> {
        &self.adapter
    }

    fn cleanup_old(&self, node_name: &str, before: DateTime<Utc>) -> Result<()> {
        self.adapter.cleanup_old(node_name, before).map_err(|err| {
            error!(error = %err, node_name, "Failed to cleanup old node minute metrics");
            err
        })
    }
}
