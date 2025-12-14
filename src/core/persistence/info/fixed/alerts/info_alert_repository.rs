use crate::core::persistence::info::fixed::info_fixed_fs_adapter_trait::InfoFixedFsAdapterTrait;

use super::info_alert_api_repository_trait::InfoAlertApiRepository;
use super::info_alert_entity::InfoAlertEntity;
use super::info_alert_fs_adapter::InfoAlertFsAdapter;

pub struct InfoAlertRepository {
    adapter: InfoAlertFsAdapter,
}

impl InfoAlertRepository {
    pub fn new() -> Self {
        Self {
            adapter: InfoAlertFsAdapter::new(),
        }
    }
}

impl InfoAlertApiRepository for InfoAlertRepository {
    fn fs_adapter(&self) -> &dyn InfoFixedFsAdapterTrait<InfoAlertEntity> {
        &self.adapter
    }
}
