use anyhow::Result;
use serde_json::Value;
use validator::Validate;

use crate::core::persistence::info::fixed::alerts::info_alert_api_repository_trait::InfoAlertApiRepository;
use crate::core::persistence::info::fixed::alerts::info_alert_entity::InfoAlertEntity;
use crate::core::persistence::info::fixed::alerts::info_alert_repository::InfoAlertRepository;
use crate::core::persistence::info::fixed::info_fixed_fs_adapter_trait::InfoFixedFsAdapterTrait;
use crate::domain::info::dto::info_alert_upsert_request::InfoAlertUpsertRequest;

pub async fn get_info_alerts() -> Result<InfoAlertEntity> {
    let repo = InfoAlertRepository::new();
    get_info_alerts_with_repo(&repo).await
}

pub async fn upsert_info_alerts(req: InfoAlertUpsertRequest) -> Result<Value> {
    req.validate()?;
    let repo = InfoAlertRepository::new();
    upsert_info_alerts_with_repo(&repo, req).await
}

async fn get_info_alerts_with_repo<R: InfoAlertApiRepository>(
    repo: &R,
) -> Result<InfoAlertEntity> {
    repo.read()
}

async fn upsert_info_alerts_with_repo<R: InfoAlertApiRepository>(
    repo: &R,
    req: InfoAlertUpsertRequest,
) -> Result<Value> {
    let mut alerts = repo.read()?;
    alerts.apply_update(req);

    repo.update(&alerts)?;

    Ok(serde_json::json!({
        "message": "Alerts updated successfully",
        "updated_at": alerts.updated_at.to_rfc3339(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockInfoAlertAdapter {
        state: Mutex<InfoAlertEntity>,
    }

    impl InfoFixedFsAdapterTrait<InfoAlertEntity> for MockInfoAlertAdapter {
        fn new() -> Self where Self: Sized {
            Self::default()
        }

        fn read(&self) -> Result<InfoAlertEntity> {
            Ok(self.state.lock().unwrap().clone())
        }

        fn insert(&self, data: &InfoAlertEntity) -> Result<()> {
            *self.state.lock().unwrap() = data.clone();
            Ok(())
        }

        fn update(&self, data: &InfoAlertEntity) -> Result<()> {
            self.insert(data)
        }

        fn delete(&self) -> Result<()> {
            *self.state.lock().unwrap() = InfoAlertEntity::default();
            Ok(())
        }
    }

    #[derive(Default)]
    struct MockInfoAlertRepository {
        adapter: MockInfoAlertAdapter,
    }

    impl InfoAlertApiRepository for MockInfoAlertRepository {
        fn fs_adapter(&self) -> &dyn InfoFixedFsAdapterTrait<InfoAlertEntity> {
            &self.adapter
        }
    }

    #[tokio::test]
    async fn upsert_alerts_updates_state() {
        let repo = MockInfoAlertRepository::default();
        let payload: InfoAlertUpsertRequest = serde_json::from_value(json!({
            "enable_cluster_health_alert": true,
            "global_alert_subject": "Updated Subject"
        }))
        .unwrap();

        let response = upsert_info_alerts_with_repo(&repo, payload.clone())
            .await
            .expect("upsert should succeed");

        let stored = repo.adapter.state.lock().unwrap().clone();
        assert!(stored.enable_cluster_health_alert);
        assert_eq!(stored.global_alert_subject, "Updated Subject");
        assert_eq!(
            response.get("message").and_then(|v| v.as_str()),
            Some("Alerts updated successfully")
        );
    }
}
