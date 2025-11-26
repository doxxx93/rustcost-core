use crate::core::persistence::metrics::metric_fs_adapter_base_trait::MetricFsAdapterBase;
use crate::core::persistence::metrics::k8s::container::metric_container_entity::MetricContainerEntity;
use anyhow::Result;
use chrono::{DateTime, Utc};

/// Repository trait for reading container minute metrics (API layer).
pub trait MetricContainerMinuteCollectorRepository: Send + Sync {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricContainerEntity>;

    /// Inserts one metric sample for a given container.
    fn append_row(&self, container_key: &str, data: &MetricContainerEntity, now:DateTime<Utc>) -> Result<()> {
        self.fs_adapter().append_row(container_key, data, now)
    }

}
