use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::core::persistence::info::fixed::llm::llm_provider::LlmProvider;

/// Upsert payload for LLM configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InfoLlmUpsertRequest {
    pub provider: Option<LlmProvider>,
    #[validate(url)]
    pub base_url: Option<String>,
    #[validate(length(min = 8))]
    pub token: Option<String>,
    #[validate(length(min = 2))]
    pub model: Option<String>,
    pub max_output_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub timeout_ms: Option<u64>,
    pub stream: Option<bool>,
    pub stop_sequences: Option<Vec<String>>,
    #[validate(length(min = 1))]
    pub organization: Option<String>,
    #[validate(length(min = 1))]
    pub user: Option<String>,
}
