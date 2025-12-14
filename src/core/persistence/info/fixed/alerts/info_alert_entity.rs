use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::info::dto::info_alert_upsert_request::{AlertRuleUpsertRequest, InfoAlertUpsertRequest};

use super::alert_rule_entity::AlertRuleEntity;

/// Alert delivery configuration extracted from the legacy settings file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoAlertEntity {
    /// Enable cluster-level health monitoring alerts.
    pub enable_cluster_health_alert: bool,
    /// Enable internal RustCost health alerts.
    pub enable_rustcost_health_alert: bool,
    /// Default subject line for alert notifications.
    pub global_alert_subject: String,
    /// Optional URL to include in alert messages for reference.
    pub linkback_url: Option<String>,
    /// Global list of alert email recipients.
    pub email_recipients: Vec<String>,
    /// Optional Slack webhook for alert delivery.
    pub slack_webhook_url: Option<String>,
    /// Optional Microsoft Teams webhook for alert delivery.
    pub teams_webhook_url: Option<String>,
    /// Optional Discord webhook for alert delivery.
    pub discord_webhook_url: Option<String>,
    /// Declarative alert rules evaluated against metrics.
    pub rules: Vec<AlertRuleEntity>,
    /// Configuration creation timestamp (UTC).
    pub created_at: DateTime<Utc>,
    /// Last update timestamp (UTC).
    pub updated_at: DateTime<Utc>,
    /// Version identifier for the configuration format.
    pub version: String,
}

impl Default for InfoAlertEntity {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            enable_cluster_health_alert: false,
            enable_rustcost_health_alert: false,
            global_alert_subject: "RustCost Alert".into(),
            linkback_url: None,
            email_recipients: vec![],
            slack_webhook_url: None,
            teams_webhook_url: None,
            discord_webhook_url: None,
            rules: Vec::new(),
            created_at: now,
            updated_at: now,
            version: "1.0.0".into(),
        }
    }
}

impl InfoAlertEntity {
    pub fn apply_update(&mut self, req: InfoAlertUpsertRequest) {
        if let Some(v) = req.enable_cluster_health_alert {
            self.enable_cluster_health_alert = v;
        }
        if let Some(v) = req.enable_rustcost_health_alert {
            self.enable_rustcost_health_alert = v;
        }
        if let Some(v) = req.global_alert_subject {
            self.global_alert_subject = v;
        }
        if let Some(v) = req.email_recipients {
            self.email_recipients = v;
        }

        if let Some(v) = normalize_string_opt(req.linkback_url) {
            self.linkback_url = v;
        }
        if let Some(v) = normalize_string_opt(req.slack_webhook_url) {
            self.slack_webhook_url = v;
        }
        if let Some(v) = normalize_string_opt(req.teams_webhook_url) {
            self.teams_webhook_url = v;
        }
        if let Some(v) = normalize_string_opt(req.discord_webhook_url) {
            self.discord_webhook_url = v;
        }

        if let Some(v) = req.rules {
            self.rules = v.into_iter().map(AlertRuleEntity::from).collect();
        }

        self.updated_at = Utc::now();
    }
}

fn normalize_string_opt(v: Option<String>) -> Option<Option<String>> {
    match v {
        Some(s) if s.trim().is_empty() => Some(None),
        Some(s) => Some(Some(s)),
        None => None,
    }
}
