use anyhow::Result;
use serde_json::Value;
use validator::Validate;

use crate::core::persistence::info::fixed::llm::info_llm_api_repository_trait::InfoLlmApiRepository;
use crate::core::persistence::info::fixed::llm::info_llm_entity::InfoLlmEntity;
use crate::core::persistence::info::fixed::llm::info_llm_repository::InfoLlmRepository;
use crate::domain::info::dto::info_llm_upsert_request::InfoLlmUpsertRequest;

pub async fn get_info_llm() -> Result<InfoLlmEntity> {
    let repo = InfoLlmRepository::new();
    get_info_llm_with_repo(&repo).await
}

pub async fn upsert_info_llm(req: InfoLlmUpsertRequest) -> Result<Value> {
    req.validate()?;
    let repo = InfoLlmRepository::new();
    upsert_info_llm_with_repo(&repo, req).await
}

async fn get_info_llm_with_repo<R: InfoLlmApiRepository>(repo: &R) -> Result<InfoLlmEntity> {
    repo.read()
}

async fn upsert_info_llm_with_repo<R: InfoLlmApiRepository>(
    repo: &R,
    req: InfoLlmUpsertRequest,
) -> Result<Value> {
    let mut cfg = repo.read()?;
    cfg.apply_update(req);

    repo.update(&cfg)?;

    Ok(serde_json::json!({
        "message": "LLM settings updated successfully",
        "updated_at": cfg.updated_at.to_rfc3339(),
        "provider": cfg.provider.as_code(),
        "model": cfg.model,
        "token": cfg.masked_token(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::persistence::info::fixed::info_fixed_fs_adapter_trait::InfoFixedFsAdapterTrait;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockInfoLlmAdapter {
        state: Mutex<InfoLlmEntity>,
    }

    impl InfoFixedFsAdapterTrait<InfoLlmEntity> for MockInfoLlmAdapter {
        fn new() -> Self where Self: Sized {
            Self::default()
        }

        fn read(&self) -> Result<InfoLlmEntity> {
            Ok(self.state.lock().unwrap().clone())
        }

        fn insert(&self, data: &InfoLlmEntity) -> Result<()> {
            *self.state.lock().unwrap() = data.clone();
            Ok(())
        }

        fn update(&self, data: &InfoLlmEntity) -> Result<()> {
            self.insert(data)
        }

        fn delete(&self) -> Result<()> {
            *self.state.lock().unwrap() = InfoLlmEntity::default();
            Ok(())
        }
    }

    #[derive(Default)]
    struct MockInfoLlmRepository {
        adapter: MockInfoLlmAdapter,
    }

    impl InfoLlmApiRepository for MockInfoLlmRepository {
        fn fs_adapter(&self) -> &dyn InfoFixedFsAdapterTrait<InfoLlmEntity> {
            &self.adapter
        }
    }

    #[tokio::test]
    async fn upsert_llm_updates_state() {
        let repo = MockInfoLlmRepository::default();
        let payload: InfoLlmUpsertRequest = serde_json::from_value(serde_json::json!({
            "provider": "grok",
            "model": "grok-1"
        }))
        .unwrap();

        let response = upsert_info_llm_with_repo(&repo, payload.clone())
            .await
            .expect("upsert should succeed");

        let stored = repo.adapter.state.lock().unwrap().clone();
        assert_eq!(stored.model.as_deref(), Some("grok-1"));
        assert_eq!(stored.provider.as_code(), "GROK");
        assert_eq!(
            response.get("message").and_then(|v| v.as_str()),
            Some("LLM settings updated successfully")
        );
    }
}
