use crate::core::persistence::metrics::k8s::pod::metric_pod_entity::MetricPodEntity;
use crate::core::persistence::metrics::k8s::pod::minute::metric_pod_minute_api_repository_trait::MetricPodMinuteApiRepository;
use crate::core::persistence::metrics::k8s::pod::minute::metric_pod_minute_collector_repository_trait::MetricPodMinuteCollectorRepository;
use crate::core::persistence::metrics::k8s::pod::minute::metric_pod_minute_fs_adapter::MetricPodMinuteFsAdapter;
use crate::core::persistence::metrics::k8s::pod::minute::metric_pod_minute_processor_repository_trait::MetricPodMinuteProcessorRepository;
use crate::core::persistence::metrics::k8s::pod::minute::metric_pod_minute_retention_repository_traits::MetricPodMinuteRetentionRepository;
use crate::core::persistence::metrics::metric_fs_adapter_base_trait::MetricFsAdapterBase;
use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::error;
use crate::core::persistence::metrics::k8s::node::day::metric_node_day_api_repository_trait::MetricNodeDayApiRepository;
use crate::core::persistence::metrics::k8s::node::day::metric_node_day_repository::MetricNodeDayRepository;
use crate::core::persistence::metrics::k8s::node::metric_node_entity::MetricNodeEntity;
use crate::domain::common::service::MetricRowRepository;

pub struct MetricPodMinuteRepository {
    adapter: MetricPodMinuteFsAdapter,
}

impl MetricPodMinuteRepository {
    pub fn new() -> Self {
        Self {
            adapter: MetricPodMinuteFsAdapter,
        }
    }
}

impl MetricRowRepository<MetricPodEntity> for MetricPodMinuteRepository {
    fn get_row_between(
        &self,
        object_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<MetricPodEntity>> {
        MetricPodMinuteApiRepository::get_row_between(self, start, end, object_name, None, None)
    }
}

impl Default for MetricPodMinuteRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricPodMinuteApiRepository for MetricPodMinuteRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricPodEntity> {
        &self.adapter
    }

    fn get_row_between(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        pod_name: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<MetricPodEntity>> {
        self.adapter
            .get_row_between(start, end, pod_name, limit, offset)
            .map_err(|err| {
                error!(error = %err, pod_name, "Failed to read pod minute rows");
                err
            })
    }
}

impl MetricPodMinuteCollectorRepository for MetricPodMinuteRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricPodEntity> {
        &self.adapter
    }

    fn append_row(&self, pod_uid: &str, data: &MetricPodEntity, now: DateTime<Utc>) -> Result<()> {
        self.adapter.append_row(pod_uid, data, now).map_err(|err| {
            error!(error = %err, pod_uid, "Failed to append pod minute row");
            err
        })
    }
}

impl MetricPodMinuteProcessorRepository for MetricPodMinuteRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricPodEntity> {
        &self.adapter
    }
}

impl MetricPodMinuteRetentionRepository for MetricPodMinuteRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricPodEntity> {
        &self.adapter
    }

    fn cleanup_old(&self, pod_uid: &str, before: DateTime<Utc>) -> Result<()> {
        self.adapter.cleanup_old(pod_uid, before).map_err(|err| {
            error!(error = %err, pod_uid, "Failed to cleanup old pod minute metrics");
            err
        })
    }
}
