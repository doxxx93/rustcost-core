use axum::extract::State;
use axum::Json;
use serde_json::Value;

use crate::api::util::json::to_json;
use crate::api::dto::ApiResponse;
use crate::app_state::AppState;
use crate::core::persistence::info::fixed::alerts::info_alert_entity::InfoAlertEntity;
use crate::domain::info::dto::info_alert_upsert_request::InfoAlertUpsertRequest;
use crate::errors::AppError;

pub struct InfoAlertController;

impl InfoAlertController {
    pub async fn get_info_alerts(
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<InfoAlertEntity>>, AppError> {
        to_json(state.info_service.get_info_alerts().await)
    }

    pub async fn upsert_info_alerts(
        State(state): State<AppState>,
        Json(payload): Json<InfoAlertUpsertRequest>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.info_service.upsert_info_alerts(payload).await)
    }
}
