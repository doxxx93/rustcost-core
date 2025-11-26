use crate::core::persistence::metrics::k8s::container::metric_container_entity::MetricContainerEntity;
use crate::core::persistence::metrics::k8s::container::minute::metric_container_minute_api_repository_trait::MetricContainerMinuteApiRepository;
use crate::core::persistence::metrics::k8s::container::minute::metric_container_minute_collector_repository_trait::MetricContainerMinuteCollectorRepository;
use crate::core::persistence::metrics::k8s::container::minute::metric_container_minute_fs_adapter::MetricContainerMinuteFsAdapter;
use crate::core::persistence::metrics::k8s::container::minute::metric_container_minute_processor_repository_trait::MetricContainerMinuteProcessorRepository;
use crate::core::persistence::metrics::k8s::container::minute::metric_container_minute_retention_repository_traits::MetricContainerMinuteRetentionRepository;
use crate::core::persistence::metrics::metric_fs_adapter_base_trait::MetricFsAdapterBase;
use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::error;

/// Repository for container minute metrics that bridges the traits and FS adapter.
pub struct MetricContainerMinuteRepository {
    adapter: MetricContainerMinuteFsAdapter,
}

impl MetricContainerMinuteRepository {
    pub fn new() -> Self {
        Self {
            adapter: MetricContainerMinuteFsAdapter,
        }
    }
}

impl Default for MetricContainerMinuteRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricContainerMinuteApiRepository for MetricContainerMinuteRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricContainerEntity> {
        &self.adapter
    }

    fn get_row_between(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        container_key: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<MetricContainerEntity>> {
        self.adapter
            .get_row_between(start, end, container_key, limit, offset)
            .map_err(|err| {
                error!(error = %err, container_key, "Failed to read container minute rows");
                err
            })
    }
}

impl MetricContainerMinuteCollectorRepository for MetricContainerMinuteRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricContainerEntity> {
        &self.adapter
    }

    fn append_row(&self, container_key: &str, data: &MetricContainerEntity, now: DateTime<Utc>) -> Result<()> {
        self.adapter.append_row(container_key, data, now).map_err(|err| {
            error!(error = %err, container_key, "Failed to append container minute row");
            err
        })
    }
}

impl MetricContainerMinuteProcessorRepository for MetricContainerMinuteRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricContainerEntity> {
        &self.adapter
    }
}

impl MetricContainerMinuteRetentionRepository for MetricContainerMinuteRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricContainerEntity> {
        &self.adapter
    }

    fn cleanup_old(&self, container_key: &str, before: DateTime<Utc>) -> Result<()> {
        self.adapter.cleanup_old(container_key, before).map_err(|err| {
            error!(error = %err, container_key, "Failed to cleanup old container minute metrics");
            err
        })
    }
}
