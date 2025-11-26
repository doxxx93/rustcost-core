use crate::core::persistence::metrics::k8s::container::metric_container_entity::MetricContainerEntity;
use crate::core::persistence::metrics::k8s::container::hour::metric_container_hour_api_repository_trait::MetricContainerHourApiRepository;
use crate::core::persistence::metrics::k8s::container::hour::metric_container_hour_fs_adapter::MetricContainerHourFsAdapter;
use crate::core::persistence::metrics::k8s::container::hour::metric_container_hour_processor_repository_trait::MetricContainerHourProcessorRepository;
use crate::core::persistence::metrics::k8s::container::hour::metric_container_hour_retention_repository_traits::MetricContainerHourRetentionRepository;
use crate::core::persistence::metrics::metric_fs_adapter_base_trait::MetricFsAdapterBase;
use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::error;

pub struct MetricContainerHourRepository {
    adapter: MetricContainerHourFsAdapter,
}

impl MetricContainerHourRepository {
    pub fn new() -> Self {
        Self {
            adapter: MetricContainerHourFsAdapter,
        }
    }
}

impl Default for MetricContainerHourRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricContainerHourApiRepository for MetricContainerHourRepository {
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
                error!(error = %err, container_key, "Failed to read container hour rows");
                err
            })
    }
}

impl MetricContainerHourProcessorRepository for MetricContainerHourRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricContainerEntity> {
        &self.adapter
    }

    fn append_row_aggregated(&self, container_key: &str, start: DateTime<Utc>, end: DateTime<Utc>, now: DateTime<Utc>) -> Result<()> {
        todo!()
    }
}

impl MetricContainerHourRetentionRepository for MetricContainerHourRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricContainerEntity> {
        &self.adapter
    }

    fn cleanup_old(&self, container_key: &str, before: DateTime<Utc>) -> Result<()> {
        self.adapter.cleanup_old(container_key, before).map_err(|err| {
            error!(error = %err, container_key, "Failed to cleanup old container hour metrics");
            err
        })
    }
}
