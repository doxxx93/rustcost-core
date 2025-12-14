use crate::core::persistence::info::fixed::info_fixed_fs_adapter_trait::InfoFixedFsAdapterTrait;
use super::info_alert_entity::InfoAlertEntity;

/// API-facing repository abstraction for alert settings.
pub trait InfoAlertApiRepository {
    fn fs_adapter(&self) -> &dyn InfoFixedFsAdapterTrait<InfoAlertEntity>;

    fn read(&self) -> anyhow::Result<InfoAlertEntity> {
        self.fs_adapter().read()
    }

    fn update(&self, settings: &InfoAlertEntity) -> anyhow::Result<()> {
        self.fs_adapter().update(settings)
    }
}
