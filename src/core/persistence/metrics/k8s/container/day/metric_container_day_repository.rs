use crate::core::persistence::metrics::k8s::container::day::metric_container_day_api_repository_trait::MetricContainerDayApiRepository;
use crate::core::persistence::metrics::k8s::container::day::metric_container_day_fs_adapter::MetricContainerDayFsAdapter;
use crate::core::persistence::metrics::k8s::container::day::metric_container_day_processor_repository_trait::MetricContainerDayProcessorRepository;
use crate::core::persistence::metrics::k8s::container::day::metric_container_day_retention_repository_traits::MetricContainerDayRetentionRepository;
use crate::core::persistence::metrics::k8s::container::metric_container_entity::MetricContainerEntity;
use crate::core::persistence::metrics::metric_fs_adapter_base_trait::MetricFsAdapterBase;
use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::error;

pub struct MetricContainerDayRepository {
    adapter: MetricContainerDayFsAdapter,
}

impl MetricContainerDayRepository {
    pub fn new() -> Self {
        Self {
            adapter: MetricContainerDayFsAdapter,
        }
    }
}

impl Default for MetricContainerDayRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricContainerDayApiRepository for MetricContainerDayRepository {
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
                error!(error = %err, container_key, "Failed to read container day rows");
                err
            })
    }
}

impl MetricContainerDayProcessorRepository for MetricContainerDayRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricContainerEntity> {
        &self.adapter
    }

    fn append_row_aggregated(&self, container_key: &str, start: DateTime<Utc>, end: DateTime<Utc>, now: DateTime<Utc>) -> Result<()> {
        todo!()
    }
}

impl MetricContainerDayRetentionRepository for MetricContainerDayRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricContainerEntity> {
        &self.adapter
    }

    fn cleanup_old(&self, container_key: &str, before: DateTime<Utc>) -> Result<()> {
        self.adapter.cleanup_old(container_key, before).map_err(|err| {
            error!(error = %err, container_key, "Failed to cleanup old container day metrics");
            err
        })
    }
}
