use axum::extract::{State};
use axum::Json;
use serde_json::Value;

use crate::api::util::json::to_json;
use crate::api::dto::ApiResponse;
use crate::app_state::AppState;
use crate::core::persistence::info::fixed::setting::info_setting_entity::InfoSettingEntity;
use crate::domain::info::dto::info_setting_upsert_request::InfoSettingUpsertRequest;
use crate::errors::AppError;

pub struct InfoSettingController;

impl InfoSettingController {
    pub async fn get_info_settings(
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<InfoSettingEntity>>, AppError> {
        to_json(state.info_service.get_info_settings().await)
    }

    pub async fn upsert_info_settings(
        State(state): State<AppState>,
        Json(payload): Json<InfoSettingUpsertRequest>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.info_service.upsert_info_settings(payload).await)
    }
}
