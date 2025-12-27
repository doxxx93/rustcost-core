use axum::extract::State;
use axum::Json;
use serde_json::Value;

use crate::api::dto::ApiResponse;
use crate::api::util::json::to_json;
use crate::app_state::AppState;
use crate::core::persistence::info::fixed::llm::info_llm_entity::InfoLlmEntity;
use crate::domain::info::dto::info_llm_upsert_request::InfoLlmUpsertRequest;
use crate::errors::AppError;

pub struct InfoLlmController;

impl InfoLlmController {
    pub async fn get_info_llm(
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<InfoLlmEntity>>, AppError> {
        to_json(state.info_service.get_info_llm().await)
    }

    pub async fn upsert_info_llm(
        State(state): State<AppState>,
        Json(payload): Json<InfoLlmUpsertRequest>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.info_service.upsert_info_llm(payload).await)
    }
}
