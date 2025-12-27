use crate::core::persistence::info::fixed::info_fixed_fs_adapter_trait::InfoFixedFsAdapterTrait;

use super::info_llm_entity::InfoLlmEntity;

/// API-facing repository abstraction for LLM settings.
pub trait InfoLlmApiRepository {
    fn fs_adapter(&self) -> &dyn InfoFixedFsAdapterTrait<InfoLlmEntity>;

    fn read(&self) -> anyhow::Result<InfoLlmEntity> {
        self.fs_adapter().read()
    }

    fn update(&self, settings: &InfoLlmEntity) -> anyhow::Result<()> {
        self.fs_adapter().update(settings)
    }
}
