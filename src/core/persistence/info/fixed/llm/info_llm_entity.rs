use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::info::dto::info_llm_upsert_request::InfoLlmUpsertRequest;
use super::llm_provider::LlmProvider;

/// Configuration for outbound LLM calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoLlmEntity {
    /// Target provider.
    pub provider: LlmProvider,
    /// Optional base URL override per provider.
    pub base_url: Option<String>,
    /// Secret token or API key.
    pub token: Option<String>,
    /// Model identifier (e.g., gpt-4o, gemini-1.5-pro-latest, grok-1).
    pub model: Option<String>,
    /// Hard limit on response tokens.
    pub max_output_tokens: Option<u32>,
    /// Temperature for sampling (0-2).
    pub temperature: Option<f32>,
    /// Nucleus sampling probability.
    pub top_p: Option<f32>,
    /// Top-k sampling (Gemini).
    pub top_k: Option<u32>,
    /// Penalize repetition (OpenAI/Grok).
    pub presence_penalty: Option<f32>,
    /// Penalize frequency (OpenAI/Grok).
    pub frequency_penalty: Option<f32>,
    /// Request timeout in milliseconds.
    pub timeout_ms: Option<u64>,
    /// Stream responses if supported.
    pub stream: bool,
    /// Stop sequences to end generation.
    pub stop_sequences: Option<Vec<String>>,
    /// Optional org/project identifier (OpenAI/Grok).
    pub organization: Option<String>,
    /// Optional user identifier to attribute requests.
    pub user: Option<String>,
    /// Configuration creation timestamp (UTC).
    pub created_at: DateTime<Utc>,
    /// Last update timestamp (UTC).
    pub updated_at: DateTime<Utc>,
    /// Version identifier for the configuration format.
    pub version: String,
}

impl Default for InfoLlmEntity {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            provider: LlmProvider::Gpt,
            base_url: None,
            token: None,
            model: Some("gpt-4o-mini".into()),
            max_output_tokens: Some(2048),
            temperature: Some(0.2),
            top_p: None,
            top_k: None,
            presence_penalty: None,
            frequency_penalty: None,
            timeout_ms: Some(30_000),
            stream: false,
            stop_sequences: None,
            organization: None,
            user: None,
            created_at: now,
            updated_at: now,
            version: "1.0.0".into(),
        }
    }
}

impl InfoLlmEntity {
    pub fn apply_update(&mut self, req: InfoLlmUpsertRequest) {
        if let Some(v) = req.provider {
            self.provider = v;
        }

        if let Some(v) = req.base_url {
            self.base_url = normalize_string(v);
        }

        if let Some(v) = req.token {
            self.token = normalize_string(v);
        }

        if let Some(v) = req.model {
            self.model = normalize_string(v);
        }

        if let Some(v) = req.max_output_tokens {
            self.max_output_tokens = Some(v);
        }

        if let Some(v) = req.temperature {
            self.temperature = Some(v);
        }

        if let Some(v) = req.top_p {
            self.top_p = Some(v);
        }

        if let Some(v) = req.top_k {
            self.top_k = Some(v);
        }

        if let Some(v) = req.presence_penalty {
            self.presence_penalty = Some(v);
        }

        if let Some(v) = req.frequency_penalty {
            self.frequency_penalty = Some(v);
        }

        if let Some(v) = req.timeout_ms {
            self.timeout_ms = Some(v);
        }

        if let Some(v) = req.stream {
            self.stream = v;
        }

        if let Some(v) = req.stop_sequences {
            let cleaned: Vec<String> = v
                .into_iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            self.stop_sequences = if cleaned.is_empty() {
                None
            } else {
                Some(cleaned)
            };
        }

        if let Some(v) = req.organization {
            self.organization = normalize_string(v);
        }

        if let Some(v) = req.user {
            self.user = normalize_string(v);
        }

        self.updated_at = Utc::now();
    }

    /// Mask the token for safe display (keeps last 4 chars).
    pub fn masked_token(&self) -> Option<String> {
        self.token.as_ref().map(|t| {
            if t.len() <= 8 {
                "***".into()
            } else {
                let tail = &t[t.len().saturating_sub(4)..];
                format!("***{}", tail)
            }
        })
    }
}

fn normalize_string(v: String) -> Option<String> {
    let s = v.trim();
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}
