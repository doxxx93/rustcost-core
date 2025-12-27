use axum::{routing::post, Router};

use crate::api::controller::llm::LlmController;
use crate::app_state::AppState;

pub fn llm_routes() -> Router<AppState> {
    Router::new()
        .route("/chat", post(LlmController::chat))
        .route("/chat-with-context", post(LlmController::chat_with_context))
}
