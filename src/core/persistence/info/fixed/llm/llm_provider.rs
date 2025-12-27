use serde::{Deserialize, Serialize};

/// Supported LLM providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmProvider {
    #[serde(rename = "gpt")]
    Gpt,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "grok")]
    Grok,
    #[serde(rename = "huggingface")]
    HuggingFace,
}

impl LlmProvider {
    pub fn as_code(&self) -> &'static str {
        match self {
            LlmProvider::Gpt => "GPT",
            LlmProvider::Gemini => "GEMINI",
            LlmProvider::Grok => "GROK",
            LlmProvider::HuggingFace => "HUGGINGFACE",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_ascii_uppercase().as_str() {
            "GPT" | "OPENAI" => Some(LlmProvider::Gpt),
            "GEMINI" | "GOOGLE" => Some(LlmProvider::Gemini),
            "GROK" | "XAI" => Some(LlmProvider::Grok),
            "HUGGINGFACE" | "HF" => Some(LlmProvider::HuggingFace),
            _ => None,
        }
    }
}
