use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, Utc};

use crate::core::persistence::info::fixed::alerts::alert_rule_entity::{
    AlertMetricType, AlertOperator, AlertRuleEntity,
};

#[derive(Debug, Clone, Default)]
pub struct AlertMetricSnapshot {
    pub cpu_usage_percent: Option<f64>,
    pub memory_usage_percent: Option<f64>,
    pub disk_usage_percent: Option<f64>,
    pub gpu_usage_percent: Option<f64>,
}

#[derive(Debug, Default)]
struct RuleState {
    active_since: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct EvaluateOutcome {
    pub triggered: Vec<AlertRuleEntity>,
    pub active_conditions: HashSet<String>,
}

/// Stateful evaluator to track rule durations between metric polls.
#[derive(Debug, Default)]
pub struct AlertRuleEvaluator {
    states: HashMap<String, RuleState>,
}

impl AlertRuleEvaluator {
    /// Evaluates rules against the current metrics snapshot.
    /// Returns rules whose conditions have been satisfied for at least `for_duration_sec`.
    pub fn evaluate(
        &mut self,
        rules: &[AlertRuleEntity],
        metrics: &AlertMetricSnapshot,
        now: DateTime<Utc>,
    ) -> EvaluateOutcome {
        let valid_ids: HashSet<String> = rules.iter().map(|r| r.id.clone()).collect();
        let mut triggered = Vec::new();
        let mut active_conditions = HashSet::new();

        for rule in rules.iter().filter(|r| r.enabled) {
            let value = Self::metric_value(rule.metric_type(), metrics);
            let state = self.states.entry(rule.id.clone()).or_default();

            let condition_met = value
                .map(|v| Self::compare(v, rule.threshold, rule.operator()))
                .unwrap_or(false);

            if condition_met {
                active_conditions.insert(rule.id.clone());

                if state.active_since.is_none() {
                    state.active_since = Some(now);
                }

                let elapsed = now.signed_duration_since(state.active_since.unwrap_or(now));
                if elapsed >= Duration::seconds(rule.for_duration_sec as i64) {
                    triggered.push(rule.clone());
                }
            } else {
                state.active_since = None;
            }
        }

        self.states.retain(|id, _| valid_ids.contains(id));
        EvaluateOutcome {
            triggered,
            active_conditions,
        }
    }

    fn metric_value(metric: AlertMetricType, metrics: &AlertMetricSnapshot) -> Option<f64> {
        match metric {
            AlertMetricType::CpuUsagePercent => metrics.cpu_usage_percent,
            AlertMetricType::MemoryUsagePercent => metrics.memory_usage_percent,
            AlertMetricType::DiskUsagePercent => metrics.disk_usage_percent,
            AlertMetricType::GpuUsagePercent => metrics.gpu_usage_percent,
        }
    }

    fn compare(value: f64, threshold: f64, op: AlertOperator) -> bool {
        match op {
            AlertOperator::GreaterThan => value > threshold,
            AlertOperator::LessThan => value < threshold,
            AlertOperator::GreaterThanOrEqual => value >= threshold,
            AlertOperator::LessThanOrEqual => value <= threshold,
        }
    }
}

trait RuleAccessors {
    fn metric_type(&self) -> AlertMetricType;
    fn operator(&self) -> AlertOperator;
}

impl RuleAccessors for AlertRuleEntity {
    fn metric_type(&self) -> AlertMetricType {
        self.metric_type.clone()
    }

    fn operator(&self) -> AlertOperator {
        self.operator.clone()
    }
}
