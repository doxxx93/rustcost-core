use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use crate::app_state::AppState;
use crate::core::persistence::info::fixed::alerts::alert_rule_entity::{
    AlertMetricType, AlertRuleEntity, AlertSeverity,
};
use crate::domain::alert::alert_rule_evaluator::{AlertMetricSnapshot, AlertRuleEvaluator};
use crate::domain::alert::discord_webhook_sender::DiscordWebhookSender;
use crate::scheduler::tasks::collectors::k8s::summary_dto::Summary;

static EVALUATOR: OnceLock<Mutex<AlertRuleEvaluator>> = OnceLock::new();

pub async fn handle_alarm(
    state: &AppState,
    summary: &Summary,
    now: DateTime<Utc>,
) -> Result<()> {
    let alert_cfg = state.info_service.get_info_alerts().await?;

    let snapshot = build_snapshot(summary);

    let (triggered, active_conditions): (Vec<AlertRuleEntity>, HashSet<String>) = {
        let evaluator = EVALUATOR.get_or_init(|| Mutex::new(AlertRuleEvaluator::default()));
        let mut guard = evaluator.lock().unwrap();
        let outcome = guard.evaluate(&alert_cfg.rules, &snapshot, now);
        (outcome.triggered, outcome.active_conditions)
    };

    for rule in triggered.iter() {
        let message = format_rule_message(rule, &snapshot);
        state
            .alerts
            .fire_alert(rule.id.clone(), message.clone(), severity_str(&rule.severity))
            .await;

        if let Some(url) = alert_cfg.discord_webhook_url.as_deref() {
            let sender = DiscordWebhookSender::default();
            if let Err(err) = sender.send(url, rule, &message).await {
                tracing::warn!(error = ?err, "Failed to send Discord webhook alert");
            }
        }
    }

    for rule in alert_cfg.rules.iter().filter(|r| r.enabled) {
        if !active_conditions.contains(&rule.id) {
            state.alerts.resolve_alert(&rule.id).await;
        }
    }

    // Legacy heuristic alarms (kept until rules replace them fully)
    check_node_memory(state, summary, now).await?;
    check_fs_usage(state, summary, now).await?;
    check_pod_memory(state, summary, now).await?;

    Ok(())
}

fn build_snapshot(summary: &Summary) -> AlertMetricSnapshot {
    let mem = &summary.node.memory;
    let working = mem.working_set_bytes.or(mem.usage_bytes);
    let avail = mem.available_bytes;
    let mem_pct = match (working, avail) {
        (Some(u), Some(a)) if u + a > 0 => Some((u as f64) / (u + a) as f64 * 100.0),
        _ => None,
    };

    let disk_pct = summary
        .node
        .fs
        .as_ref()
        .and_then(|fs| match (fs.used_bytes, fs.capacity_bytes) {
            (Some(used), Some(cap)) if cap > 0 => Some((used as f64 / cap as f64) * 100.0),
            _ => None,
        });

    AlertMetricSnapshot {
        cpu_usage_percent: None,  // TODO: add node capacity to compute CPU %
        memory_usage_percent: mem_pct,
        disk_usage_percent: disk_pct,
        gpu_usage_percent: None,
    }
}

fn format_rule_message(rule: &AlertRuleEntity, snapshot: &AlertMetricSnapshot) -> String {
    let value = metric_value(rule.metric_type.clone(), snapshot);
    match value {
        Some(v) => format!(
            "{}: observed {:.1}% {} (rule {} {:.1}% for {}s)",
            rule.name,
            v,
            rule.metric_type.as_code(),
            rule.operator.as_code(),
            rule.threshold,
            rule.for_duration_sec
        ),
        None => format!(
            "{} triggered (metric unavailable for display, threshold {:.1})",
            rule.name, rule.threshold
        ),
    }
}

fn metric_value(metric: AlertMetricType, snapshot: &AlertMetricSnapshot) -> Option<f64> {
    match metric {
        AlertMetricType::CpuUsagePercent => snapshot.cpu_usage_percent,
        AlertMetricType::MemoryUsagePercent => snapshot.memory_usage_percent,
        AlertMetricType::DiskUsagePercent => snapshot.disk_usage_percent,
        AlertMetricType::GpuUsagePercent => snapshot.gpu_usage_percent,
    }
}

fn severity_str(sev: &AlertSeverity) -> String {
    match sev {
        AlertSeverity::Info => "info",
        AlertSeverity::Warning => "warning",
        AlertSeverity::Critical => "critical",
    }
    .to_string()
}

async fn check_pod_memory(
    state: &AppState,
    summary: &Summary,
    _now: DateTime<Utc>,
) -> Result<()> {
    let Some(pods) = &summary.pods else {
        return Ok(());
    };

    let node_total_mem = summary
        .node
        .memory
        .usage_bytes
        .unwrap_or(0)
        + summary.node.memory.available_bytes.unwrap_or(0);

    if node_total_mem == 0 {
        // Cannot determine percentages → skip alerts gracefully
        return Ok(());
    }

    for pod in pods {
        let ws = pod.memory.working_set_bytes.unwrap_or(0);
        let pct = ws as f64 / node_total_mem as f64;

        // Stable per-pod alert ID
        let alert_id = format!("pod-mem-{}", pod.pod_ref.uid);

        // Hysteresis thresholds
        const TRIGGER: f64 = 0.80;  // pod uses > 80% of node memory
        const RESOLVE: f64 = 0.60;  // resolves when < 60%

        if pct > TRIGGER {
            state.alerts
                .fire_alert(
                    alert_id.clone(),
                    format!(
                        "Pod {}/{} using {:.1}% of node memory ({} MiB)",
                        pod.pod_ref.namespace,
                        pod.pod_ref.name,
                        pct * 100.0,
                        ws / 1024 / 1024
                    ),
                    "warning".into(),
                )
                .await;
        } else if pct < RESOLVE {
            state.alerts.resolve_alert(&alert_id).await;
        }
    }

    Ok(())
}



async fn check_fs_usage(
    state: &AppState,
    summary: &Summary,
    _now: DateTime<Utc>,
) -> Result<()> {

    let Some(fs) = &summary.node.fs else {
        tracing::warn!("Node FS metrics missing");
        return Ok(());
    };

    let (Some(cap), Some(used)) = (fs.capacity_bytes, fs.used_bytes) else {
        tracing::warn!("Node FS missing capacity or used metrics");
        return Ok(());
    };

    if cap == 0 {
        tracing::warn!("Node FS reports zero capacity — skipping check");
        return Ok(());
    }

    let pct_used = used as f64 / cap as f64;
    let pct_display = pct_used * 100.0;

    let id = "node-fs-full";

    // Hysteresis: trigger >90%, resolve <85%
    if pct_used > 0.90 {
        state.alerts
            .fire_alert(
                id.to_string(),
                format!(
                    "Node filesystem usage high: {:.1}% (used={} GiB / cap={} GiB)",
                    pct_display,
                    used / 1024 / 1024 / 1024,
                    cap / 1024 / 1024 / 1024
                ),
                "critical".into(),
            )
            .await;
    } else if pct_used < 0.85 {
        state.alerts.resolve_alert(id).await;
    } else {
        // Between 85–90% → do nothing to avoid alert flapping
    }

    Ok(())
}


async fn check_node_memory(
    state: &AppState,
    summary: &Summary,
    _now: DateTime<Utc>,
) -> Result<()> {

    let mem = &summary.node.memory;

    // Prefer working_set_bytes (much more stable)
    let working_set = mem.working_set_bytes.unwrap_or(0);

    // Optionally, use RSS or usage_bytes as fallback
    let used = mem.usage_bytes.unwrap_or(working_set);

    // available_bytes comes from cgroups and can be 0 sometimes
    let avail = mem.available_bytes.unwrap_or(0);

    // Compute pct_free safely
    let total = used + avail;

    if total == 0 {
        // Node is reporting garbage → no alert, but do not hide the issue
        tracing::warn!("Node memory stats appear invalid: used=0 avail=0");
        return Ok(());
    }

    let pct_free = avail as f64 / total as f64;

    let id = "node-low-mem";

    if pct_free < 0.10 {
        state.alerts
            .fire_alert(
                id.into(),
                format!(
                    "Node memory low: {:.1}% free (used={} MiB, avail={} MiB)",
                    pct_free * 100.0,
                    used / 1024 / 1024,
                    avail / 1024 / 1024
                ),
                "warning".into(),
            )
            .await;
    } else {
        state.alerts.resolve_alert(id).await;
    }

    Ok(())
}

// async fn check_node_cpu(
//     state: &AppState,
//     summary: &Summary,
//     _now: DateTime<Utc>,
// ) -> Result<()> {
//     let usage_nano = summary.node.cpu.usage_nano_cores.unwrap_or(0);
//
//     // Convert to millicores
//     let usage_mcores = usage_nano as f64 / 1_000_000.0;
//     let node_name = &summary.node.node_name;
//     // Retrieve node capacity (must be stored in DB or cached in AppState)
//     let capacity_mcores = state
//         .info_k8s_service
//         .get_info_k8s_node(node_name)
//         .await
//         .unwrap_or(1000); // default = 1 CPU
//
//     let pct = usage_mcores / capacity_mcores as f64;
//     let id = format!("node-high-cpu-{}", summary.node.node_name);
//
//     if pct > 0.85 {
//         state.alerts
//             .fire_alert(
//                 id.clone(),
//                 format!(
//                     "Node CPU high: {:.0}% ({}m / {}m)",
//                     pct * 100.0,
//                     usage_mcores as u64,
//                     capacity_mcores
//                 ),
//                 "warning".into(),
//             )
//             .await;
//     } else {
//         state.alerts.resolve_alert(&id).await;
//     }
//
//     Ok(())
// }
