use crate::core::persistence::metrics::k8s::pod::hour::metric_pod_hour_api_repository_trait::MetricPodHourApiRepository;
use crate::core::persistence::metrics::k8s::pod::hour::metric_pod_hour_fs_adapter::MetricPodHourFsAdapter;
use crate::core::persistence::metrics::k8s::pod::hour::metric_pod_hour_retention_repository_traits::MetricPodHourRetentionRepository;
use crate::core::persistence::metrics::k8s::pod::metric_pod_entity::MetricPodEntity;
use crate::core::persistence::metrics::metric_fs_adapter_base_trait::MetricFsAdapterBase;
use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::error;
use crate::core::persistence::metrics::k8s::node::day::metric_node_day_api_repository_trait::MetricNodeDayApiRepository;
use crate::core::persistence::metrics::k8s::node::day::metric_node_day_repository::MetricNodeDayRepository;
use crate::core::persistence::metrics::k8s::node::metric_node_entity::MetricNodeEntity;
use crate::domain::common::service::MetricRowRepository;

pub struct MetricPodHourRepository {
    adapter: MetricPodHourFsAdapter,
}

impl MetricPodHourRepository {
    pub fn new() -> Self {
        Self {
            adapter: MetricPodHourFsAdapter,
        }
    }
}

impl MetricRowRepository<MetricPodEntity> for MetricPodHourRepository {
    fn get_row_between(
        &self,
        object_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<MetricPodEntity>> {
        MetricPodHourApiRepository::get_row_between(self, start, end, object_name, None, None)
    }
}

impl Default for MetricPodHourRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricPodHourApiRepository for MetricPodHourRepository {
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
                error!(error = %err, pod_name, "Failed to read pod hour rows");
                err
            })
    }
}

impl MetricPodHourRetentionRepository for MetricPodHourRepository {
    fn fs_adapter(&self) -> &dyn MetricFsAdapterBase<MetricPodEntity> {
        &self.adapter
    }

    fn cleanup_old(&self, pod_uid: &str, before: DateTime<Utc>) -> Result<()> {
        self.adapter.cleanup_old(pod_uid, before).map_err(|err| {
            error!(error = %err, pod_uid, "Failed to cleanup old pod hour metrics");
            err
        })
    }
}
