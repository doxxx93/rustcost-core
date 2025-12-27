use axum::extract::State;
use axum::Json;
use serde_json::Value;

use crate::api::dto::ApiResponse;
use crate::api::util::json::to_json;
use crate::app_state::AppState;
use crate::domain::llm::dto::llm_chat_request::LlmChatRequest;
use crate::domain::llm::dto::llm_chat_with_context_request::LlmChatWithContextRequest;
use crate::errors::AppError;

pub struct LlmController;

impl LlmController {
    pub async fn chat(
        State(state): State<AppState>,
        Json(payload): Json<LlmChatRequest>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.llm_service.chat(payload).await)
    }

    pub async fn chat_with_context(
        State(state): State<AppState>,
        Json(payload): Json<LlmChatWithContextRequest>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.llm_service.chat_with_context(payload).await)
    }
}
