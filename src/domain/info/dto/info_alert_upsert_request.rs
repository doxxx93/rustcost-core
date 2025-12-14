use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::core::persistence::info::fixed::alerts::alert_rule_entity::{
    AlertMetricType, AlertOperator, AlertRuleEntity, AlertSeverity,
};

/// Upsert payload for alert configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InfoAlertUpsertRequest {
    /// Enable cluster-level health monitoring alerts.
    pub enable_cluster_health_alert: Option<bool>,

    /// Enable internal RustCost health alerts.
    pub enable_rustcost_health_alert: Option<bool>,

    /// Default subject line for alert notifications.
    #[validate(length(min = 1, max = 100))]
    pub global_alert_subject: Option<String>,

    /// Optional URL to include in alert messages for reference.
    #[validate(url)]
    pub linkback_url: Option<String>,

    /// Global list of alert email recipients.
    pub email_recipients: Option<Vec<String>>,

    /// Optional Slack webhook for alert delivery.
    #[validate(url)]
    pub slack_webhook_url: Option<String>,

    /// Optional Microsoft Teams webhook for alert delivery.
    #[validate(url)]
    pub teams_webhook_url: Option<String>,

    /// Optional Discord webhook for alert delivery.
    #[validate(url)]
    pub discord_webhook_url: Option<String>,

    /// Declarative alert rules.
    #[validate(nested)]
    pub rules: Option<Vec<AlertRuleUpsertRequest>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AlertRuleUpsertRequest {
    #[validate(length(min = 1))]
    pub id: String,
    #[validate(length(min = 1))]
    pub name: String,
    pub metric_type: AlertMetricType,
    pub operator: AlertOperator,
    pub threshold: f64,
    pub for_duration_sec: u64,
    pub severity: AlertSeverity,
    pub enabled: bool,
}

impl From<AlertRuleUpsertRequest> for AlertRuleEntity {
    fn from(value: AlertRuleUpsertRequest) -> Self {
        Self {
            id: value.id,
            name: value.name,
            metric_type: value.metric_type,
            operator: value.operator,
            threshold: value.threshold,
            for_duration_sec: value.for_duration_sec,
            severity: value.severity,
            enabled: value.enabled,
        }
    }
}
