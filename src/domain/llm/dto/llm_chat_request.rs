use serde::{Deserialize, Serialize};
use validator::Validate;

/// Chat completion payload for Hugging Face router.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LlmChatRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub messages: Vec<LlmMessage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LlmMessage {
    #[validate(length(min = 1))]
    pub role: String,
    #[validate(length(min = 1))]
    pub content: String,
}
