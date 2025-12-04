use std::sync::Arc;
use chrono::Utc;

use crate::core::state::runtime::alerts::alert_runtime_state::{AlertEvent, AlertRuntimeState};
use crate::core::state::runtime::alerts::alert_runtime_state_repository_trait::AlertRuntimeStateRepositoryTrait;

pub struct AlertRuntimeStateManager<R: AlertRuntimeStateRepositoryTrait> {
    pub(crate) repo: Arc<R>,
}

impl<R: AlertRuntimeStateRepositoryTrait> AlertRuntimeStateManager<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    /// Maximum alerts allowed in `WINDOW_SECONDS`
    const MAX_ALERTS_PER_WINDOW: usize = 100;
    const WINDOW_SECONDS: i64 = 60;

    pub async fn fire_alert(&self, id: String, message: String, severity: String) {
        self.repo.update(|state| {
            // Step 1 — prune old timestamps
            state.prune_old_timestamps(Self::WINDOW_SECONDS);

            // Step 2 — check recent count
            let recent = state.recent_alert_times.len();

            if recent >= Self::MAX_ALERTS_PER_WINDOW {
                // Alert storm → suppress normal alerts
                let storm_id = "alert-storm".to_string();
                let storm_msg = format!(
                    "Alert storm detected: > {} alerts in {} seconds. Suppressing new alerts.",
                    Self::MAX_ALERTS_PER_WINDOW,
                    Self::WINDOW_SECONDS
                );

                let storm_event = AlertEvent {
                    id: storm_id.clone(),
                    message: storm_msg,
                    severity: "critical".into(),
                    created_at: Utc::now(),
                    last_updated_at: Utc::now(),
                    active: true,
                };

                state.add_or_update_alert(storm_event);
                return;
            }

            // Step 3 — register timestamp
            let now = Utc::now();
            state.recent_alert_times.push_back(now);

            // Step 4 — normal alert
            let alert = AlertEvent {
                id,
                message,
                severity,
                created_at: now,
                last_updated_at: now,
                active: true,
            };

            state.add_or_update_alert(alert);
        }).await;
    }

    pub async fn resolve_alert(&self, id: &str) {
        self.repo.update(|state| {
            state.resolve_alert(id);
        }).await;
    }

    pub async fn reset(&self) {
        self.repo.set(AlertRuntimeState::default()).await;
    }

    pub async fn active_alerts(&self) -> Vec<AlertEvent> {
        let s = self.repo.get().await;
        s.active_alerts()
    }
}
