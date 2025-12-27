// src/domain/llm/service/llm_chat_service.rs
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;
use validator::Validate;
use crate::core::persistence::info::fixed::llm::info_llm_api_repository_trait::InfoLlmApiRepository;
use crate::core::persistence::info::fixed::llm::info_llm_repository::InfoLlmRepository;
use crate::core::persistence::info::fixed::llm::llm_provider::LlmProvider;
use crate::domain::info::service::{info_alerts_service, info_k8s_node_service};
use crate::domain::llm::dto::llm_chat_request::{LlmChatRequest, LlmMessage};
use crate::domain::llm::dto::llm_chat_with_context_request::LlmChatWithContextRequest;

/// Call Hugging Face router using stored LLM configuration.
pub async fn chat(payload: LlmChatRequest) -> Result<Value> {
    payload.validate()?;

    let cfg = InfoLlmRepository::new().read()?;
    if cfg.provider != LlmProvider::HuggingFace {
        return Err(anyhow!(
            "LLM provider must be set to HuggingFace to call this endpoint"
        ));
    }

    let token = cfg
        .token
        .clone()
        .ok_or_else(|| anyhow!("LLM token is missing; set it in /info/llm"))?;

    let model = payload
        .model
        .clone()
        .or_else(|| cfg.model.clone())
        .ok_or_else(|| anyhow!("Model is missing; set it in /info/llm or request payload"))?;

    let base_url = cfg
        .base_url
        .clone()
        .unwrap_or_else(|| "https://router.huggingface.co/v1".to_string());
    let trimmed = base_url.trim_end_matches('/');
    let url = if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else {
        format!("{}/chat/completions", trimmed)
    };

    let mut body = serde_json::json!({
        "model": model,
        "messages": payload.messages,
        "stream": payload.stream.unwrap_or(cfg.stream),
    });

    if let Some(v) = payload.max_tokens.or(cfg.max_output_tokens) {
        body["max_tokens"] = serde_json::json!(v);
    }
    if let Some(v) = payload.temperature.or(cfg.temperature) {
        body["temperature"] = serde_json::json!(v);
    }
    if let Some(v) = payload.top_p.or(cfg.top_p) {
        body["top_p"] = serde_json::json!(v);
    }

    let body_str = serde_json::to_string(&body).unwrap_or_else(|_| "<failed-to-serialize-body>".to_string());

    let client = Client::builder()
        .build()
        .map_err(|e| anyhow!("Failed to build HTTP client: {}", e))?;

    let resp = client
        .post(&url)
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to call Hugging Face (url={}, body={}): {}", url, body_str, e))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Hugging Face returned {}: {} (url={}, body={})", status, text, url, body_str));
    }

    let json: Value = resp
        .json()
        .await
        .map_err(|e| anyhow!("Failed to decode Hugging Face response: {} (url={}, body={})", e, url, body_str))?;

    Ok(json)
}

/// Call LLM with backend-built cluster/alert context.
pub async fn chat_with_context(payload: LlmChatWithContextRequest) -> Result<Value> {
    payload.validate()?;

    let mut context_sections = Vec::new();

    if payload.include_cluster_summary {
        if let Some(section) = build_node_summary(payload.time_window_minutes).await? {
            context_sections.push(section);
        }
    }

    if payload.include_alerts {
        if let Some(section) = build_alerts_summary().await? {
            context_sections.push(section);
        }
    }

    let include_cluster_summary = payload.include_cluster_summary;
    let include_alerts = payload.include_alerts;
    let window_label = payload.time_window_minutes.unwrap_or(15);

    let mut chat_payload: LlmChatRequest = payload.into();
    let mut messages = Vec::new();
    if !context_sections.is_empty() {
        messages.push(LlmMessage {
            role: "system".into(),
            content: context_sections.join("\n\n"),
        });
    }
    messages.extend(chat_payload.messages.clone());
    chat_payload.messages = messages;

    let model_label = chat_payload
        .model
        .clone()
        .unwrap_or_else(|| "default-from-config".to_string());

    chat(chat_payload).await.map_err(|e| {
        anyhow!(
            "LLM chat_with_context failed (model={}, include_cluster_summary={}, include_alerts={}, window_minutes={}): {}",
            model_label,
            include_cluster_summary,
            include_alerts,
            window_label,
            e
        )
    })
}

async fn build_node_summary(time_window_minutes: Option<u32>) -> Result<Option<String>> {
    use crate::api::dto::info_dto::K8sListNodeQuery;
    use crate::api::dto::metrics_dto::{CostMode, RangeQuery};
    use chrono::Utc;

    let nodes = info_k8s_node_service::list_k8s_nodes(K8sListNodeQuery::default()).await?;
    let node_names: Vec<String> = nodes
        .iter()
        .filter_map(|n| n.node_name.clone())
        .take(3)
        .collect();

    if node_names.is_empty() {
        return Ok(None);
    }

    let minutes = time_window_minutes.unwrap_or(15) as i64;
    let end = Utc::now().naive_utc();
    let start = end - chrono::Duration::minutes(minutes);

    let q = RangeQuery {
        start: Some(start),
        end: Some(end),
        granularity: None,
        limit: Some(node_names.len()),
        offset: Some(0),
        sort: None,
        mode: CostMode::Showback,
        team: None,
        service: None,
        env: None,
        namespace: None,
        labels: None,
        key: None,
    };

    let summary = crate::domain::metric::k8s::node::service::get_metric_k8s_nodes_raw_summary(
        q,
        node_names.clone(),
    )
        .await?;

    let summary_str = serde_json::to_string(&summary)?;
    Ok(Some(format!(
        "Cluster node summary ({} nodes, last {}m): {}",
        node_names.len(),
        minutes,
        trim_str(&summary_str, 1200)
    )))
}

async fn build_alerts_summary() -> Result<Option<String>> {
    let alerts = info_alerts_service::get_info_alerts().await?;
    let mut parts = Vec::new();
    parts.push(format!(
        "Cluster alerts enabled: {}",
        alerts.enable_cluster_health_alert
    ));
    parts.push(format!(
        "RustCost health alerts enabled: {}",
        alerts.enable_rustcost_health_alert
    ));
    if !alerts.email_recipients.is_empty() {
        parts.push(format!(
            "Email recipients: {}",
            alerts.email_recipients.join(", ")
        ));
    }
    let rule_count = alerts.rules.len();
    parts.push(format!("Alert rules: {}", rule_count));

    Ok(Some(format!("Alert config: {}", parts.join(" | "))))
}

fn trim_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...<truncated>", &s[..max_len])
    }
}
