use anyhow::Result;
use serde_json::Value;
use crate::core::persistence::info::fixed::setting::info_setting_api_repository_trait::InfoSettingApiRepository;
use crate::core::persistence::info::fixed::setting::info_setting_entity::InfoSettingEntity;
use crate::core::persistence::info::fixed::setting::info_setting_repository::InfoSettingRepository;
use crate::domain::info::dto::info_setting_upsert_request::InfoSettingUpsertRequest;
use validator::Validate;

pub async fn get_info_settings() -> Result<InfoSettingEntity> {
    let repo = InfoSettingRepository::new();
    get_info_settings_with_repo(&repo).await
}

pub async fn upsert_info_settings(req: InfoSettingUpsertRequest) -> Result<Value> {
    req.validate()?;
    let repo = InfoSettingRepository::new();
    upsert_info_settings_with_repo(&repo, req).await
}

async fn get_info_settings_with_repo<R: InfoSettingApiRepository>(
    repo: &R,
) -> Result<InfoSettingEntity> {
    repo.read()
}

async fn upsert_info_settings_with_repo<R: InfoSettingApiRepository>(
    repo: &R,
    req: InfoSettingUpsertRequest,
) -> Result<Value> {
    let mut settings = repo.read()?;
    settings.apply_update(req);

    repo.update(&settings)?;

    Ok(serde_json::json!({
        "message": "Settings updated successfully",
        "updated_at": settings.updated_at.to_rfc3339(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::persistence::info::fixed::info_fixed_fs_adapter_trait::InfoFixedFsAdapterTrait;
    use serde_json::json;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockInfoSettingAdapter {
        state: Mutex<InfoSettingEntity>,
    }

    impl InfoFixedFsAdapterTrait<InfoSettingEntity> for MockInfoSettingAdapter {
        fn new() -> Self where Self: Sized {
            Self::default()
        }

        fn read(&self) -> Result<InfoSettingEntity> {
            Ok(self.state.lock().unwrap().clone())
        }

        fn insert(&self, data: &InfoSettingEntity) -> Result<()> {
            *self.state.lock().unwrap() = data.clone();
            Ok(())
        }

        fn update(&self, data: &InfoSettingEntity) -> Result<()> {
            self.insert(data)
        }

        fn delete(&self) -> Result<()> {
            *self.state.lock().unwrap() = InfoSettingEntity::default();
            Ok(())
        }
    }

    #[derive(Default)]
    struct MockInfoSettingRepository {
        adapter: MockInfoSettingAdapter,
    }

    impl InfoSettingApiRepository for MockInfoSettingRepository {
        fn fs_adapter(&self) -> &dyn InfoFixedFsAdapterTrait<InfoSettingEntity> {
            &self.adapter
        }
    }

    #[tokio::test]
    async fn upsert_uses_trait_repository() {
        let repo = MockInfoSettingRepository::default();
        let payload: InfoSettingUpsertRequest = serde_json::from_value(json!({
            "language": "ja",
            "is_dark_mode": true
        }))
        .unwrap();

        let response = upsert_info_settings_with_repo(&repo, payload.clone())
            .await
            .expect("upsert should succeed");

        let stored = repo.adapter.state.lock().unwrap().clone();
        assert_eq!(stored.language, "ja");
        assert!(stored.is_dark_mode);
        assert_eq!(
            response.get("message").and_then(|v| v.as_str()),
            Some("Settings updated successfully")
        );
    }
}
