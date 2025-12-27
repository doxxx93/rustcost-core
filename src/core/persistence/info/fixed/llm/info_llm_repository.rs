use crate::core::persistence::info::fixed::info_fixed_fs_adapter_trait::InfoFixedFsAdapterTrait;

use super::info_llm_api_repository_trait::InfoLlmApiRepository;
use super::info_llm_entity::InfoLlmEntity;
use super::info_llm_fs_adapter::InfoLlmFsAdapter;

pub struct InfoLlmRepository {
    adapter: InfoLlmFsAdapter,
}

impl InfoLlmRepository {
    pub fn new() -> Self {
        Self {
            adapter: InfoLlmFsAdapter::new(),
        }
    }
}

impl InfoLlmApiRepository for InfoLlmRepository {
    fn fs_adapter(&self) -> &dyn InfoFixedFsAdapterTrait<InfoLlmEntity> {
        &self.adapter
    }
}
