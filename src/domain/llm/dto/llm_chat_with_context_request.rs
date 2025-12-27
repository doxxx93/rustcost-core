use serde::{Deserialize, Serialize};
use validator::Validate;

use super::llm_chat_request::{LlmChatRequest, LlmMessage};

/// Chat request with backend-built context.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LlmChatWithContextRequest {
    #[validate(length(min = 1))]
    pub messages: Vec<LlmMessage>,
    #[validate(length(min = 2))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Include cluster/node summary section.
    #[serde(default)]
    pub include_cluster_summary: bool,
    /// Include alert config summary.
    #[serde(default)]
    pub include_alerts: bool,
    /// Lookback window in minutes for metrics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_window_minutes: Option<u32>,
}

impl From<LlmChatWithContextRequest> for LlmChatRequest {
    fn from(value: LlmChatWithContextRequest) -> Self {
        LlmChatRequest {
            model: value.model,
            messages: value.messages,
            stream: value.stream,
            max_tokens: value.max_tokens,
            temperature: value.temperature,
            top_p: value.top_p,
        }
    }
}
