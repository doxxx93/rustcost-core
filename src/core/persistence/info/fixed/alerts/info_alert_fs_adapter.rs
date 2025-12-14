use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

use crate::core::persistence::info::fixed::info_fixed_fs_adapter_trait::InfoFixedFsAdapterTrait;
use crate::core::persistence::storage_path::{info_alert_path, info_setting_path};

use super::alert_rule_entity::{AlertMetricType, AlertOperator, AlertRuleEntity, AlertSeverity};
use super::info_alert_entity::InfoAlertEntity;

/// FS adapter for persisted alert settings.
///
/// Reads and writes a simple key-value file located at `alerts.rci`.
/// Falls back to legacy values inside `settings.rci` if the new file
/// is missing, so existing installations keep their alert config.
pub struct InfoAlertFsAdapter;

impl InfoFixedFsAdapterTrait<InfoAlertEntity> for InfoAlertFsAdapter {
    fn new() -> Self {
        Self {}
    }

    fn read(&self) -> Result<InfoAlertEntity> {
        let path = info_alert_path();
        if path.exists() {
            return Self::read_from_path(&path);
        }

        let legacy = info_setting_path();
        if legacy.exists() {
            if let Ok(entity) = Self::read_from_path(&legacy) {
                return Ok(entity);
            }
        }

        Ok(InfoAlertEntity::default())
    }

    fn insert(&self, data: &InfoAlertEntity) -> Result<()> {
        self.write(data)
    }

    fn update(&self, data: &InfoAlertEntity) -> Result<()> {
        self.write(data)
    }

    fn delete(&self) -> Result<()> {
        let path = info_alert_path();
        if path.exists() {
            fs::remove_file(&path).context("Failed to delete alerts file")?;
        }
        Ok(())
    }
}

impl InfoAlertFsAdapter {
    fn read_from_path(path: &Path) -> Result<InfoAlertEntity> {
        let file = File::open(path).context("Failed to open alerts file")?;
        let reader = BufReader::new(file);
        let mut s = InfoAlertEntity::default();
        let mut raw_rules: HashMap<String, String> = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim().to_uppercase();
                let val = val.trim();

                if key.starts_with("ALERT_RULE_") {
                    raw_rules.insert(key.clone(), val.to_string());
                }

                match key.as_str() {
                    "ENABLE_CLUSTER_HEALTH_ALERT" => {
                        s.enable_cluster_health_alert = val.eq_ignore_ascii_case("true")
                    }
                    "ENABLE_RUSTCOST_HEALTH_ALERT" => {
                        s.enable_rustcost_health_alert = val.eq_ignore_ascii_case("true")
                    }
                    "GLOBAL_ALERT_SUBJECT" => s.global_alert_subject = val.to_string(),
                    "LINKBACK_URL" => {
                        s.linkback_url = if val.is_empty() {
                            None
                        } else {
                            Some(val.to_string())
                        }
                    }
                    "EMAIL_RECIPIENTS" => {
                        s.email_recipients = val
                            .split(',')
                            .map(|v| v.trim().to_string())
                            .filter(|v| !v.is_empty())
                            .collect();
                    }
                    "SLACK_WEBHOOK_URL" => {
                        s.slack_webhook_url = if val.is_empty() {
                            None
                        } else {
                            Some(val.to_string())
                        }
                    }
                    "TEAMS_WEBHOOK_URL" => {
                        s.teams_webhook_url = if val.is_empty() {
                            None
                        } else {
                            Some(val.to_string())
                        }
                    }
                    "DISCORD_WEBHOOK_URL" => {
                        s.discord_webhook_url = if val.is_empty() {
                            None
                        } else {
                            Some(val.to_string())
                        }
                    }
                    "CREATED_AT" => {
                        if let Ok(dt) = val.parse::<DateTime<Utc>>() {
                            s.created_at = dt;
                        }
                    }
                    "UPDATED_AT" => {
                        if let Ok(dt) = val.parse::<DateTime<Utc>>() {
                            s.updated_at = dt;
                        }
                    }
                    "VERSION" => s.version = val.to_string(),
                    _ => {}
                }
            }
        }

        s.rules = Self::parse_rules(&raw_rules);
        Ok(s)
    }

    fn write(&self, data: &InfoAlertEntity) -> Result<()> {
        use std::io::Write;

        let path = info_alert_path();

        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).context("Failed to create alerts directory")?;
        }

        let tmp_path = path.with_extension("rci.tmp");
        let mut f = File::create(&tmp_path).context("Failed to create temp alerts file")?;

        writeln!(f, "ALERT_RULE_COUNT:{}", data.rules.len())?;
        for (idx, rule) in data.rules.iter().enumerate() {
            writeln!(f, "ALERT_RULE_{}_ID:{}", idx, rule.id)?;
            writeln!(f, "ALERT_RULE_{}_NAME:{}", idx, rule.name)?;
            writeln!(f, "ALERT_RULE_{}_METRIC:{}", idx, rule.metric_type.as_code())?;
            writeln!(f, "ALERT_RULE_{}_OPERATOR:{}", idx, rule.operator.as_code())?;
            writeln!(f, "ALERT_RULE_{}_THRESHOLD:{}", idx, rule.threshold)?;
            writeln!(f, "ALERT_RULE_{}_FOR_SEC:{}", idx, rule.for_duration_sec)?;
            writeln!(f, "ALERT_RULE_{}_SEVERITY:{}", idx, rule.severity.as_code())?;
            writeln!(f, "ALERT_RULE_{}_ENABLED:{}", idx, rule.enabled)?;
        }

        writeln!(f, "ENABLE_CLUSTER_HEALTH_ALERT:{}", data.enable_cluster_health_alert)?;
        writeln!(f, "ENABLE_RUSTCOST_HEALTH_ALERT:{}", data.enable_rustcost_health_alert)?;
        writeln!(f, "GLOBAL_ALERT_SUBJECT:{}", data.global_alert_subject)?;
        writeln!(f, "LINKBACK_URL:{}", data.linkback_url.clone().unwrap_or_default())?;
        writeln!(f, "EMAIL_RECIPIENTS:{}", data.email_recipients.join(","))?;
        writeln!(f, "SLACK_WEBHOOK_URL:{}", data.slack_webhook_url.clone().unwrap_or_default())?;
        writeln!(f, "TEAMS_WEBHOOK_URL:{}", data.teams_webhook_url.clone().unwrap_or_default())?;
        writeln!(f, "DISCORD_WEBHOOK_URL:{}", data.discord_webhook_url.clone().unwrap_or_default())?;
        writeln!(f, "CREATED_AT:{}", data.created_at.to_rfc3339())?;
        writeln!(f, "UPDATED_AT:{}", data.updated_at.to_rfc3339())?;
        writeln!(f, "VERSION:{}", data.version)?;

        f.flush()?;
        f.sync_all().context("Failed to sync temp alerts file")?;

        fs::rename(&tmp_path, &path).context("Failed to finalize alerts file")?;

        #[cfg(unix)]
        if let Some(dir) = path.parent() {
            use std::os::unix::fs::FileExt as _;
            let dir_file = File::open(dir).context("Failed to open alerts directory")?;
            dir_file.sync_all().context("Failed to sync alerts directory")?;
        }

        Ok(())
    }

    fn parse_rules(raw: &HashMap<String, String>) -> Vec<AlertRuleEntity> {
        let count = raw
            .get("ALERT_RULE_COUNT")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let mut rules = Vec::with_capacity(count);

        for idx in 0..count {
            let prefix = format!("ALERT_RULE_{}_", idx);
            let get = |suffix: &str| -> Option<String> {
                raw.get(&(prefix.clone() + suffix)).map(|v| v.to_string())
            };

            let id = get("ID").unwrap_or_else(|| format!("rule-{}", idx));
            let name = get("NAME").unwrap_or_else(|| id.clone());
            let metric = get("METRIC")
                .and_then(AlertMetricType::from_code)
                .unwrap_or(AlertMetricType::CpuUsagePercent);
            let operator = get("OPERATOR")
                .and_then(AlertOperator::from_code)
                .unwrap_or(AlertOperator::GreaterThan);
            let threshold = get("THRESHOLD")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            let for_duration_sec = get("FOR_SEC")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(0);
            let severity = get("SEVERITY")
                .and_then(AlertSeverity::from_code)
                .unwrap_or(AlertSeverity::Warning);
            let enabled = get("ENABLED")
                .map(|v| v.eq_ignore_ascii_case("true"))
                .unwrap_or(true);

            rules.push(AlertRuleEntity {
                id,
                name,
                metric_type: metric,
                operator,
                threshold,
                for_duration_sec,
                severity,
                enabled,
            });
        }

        rules
    }
}
