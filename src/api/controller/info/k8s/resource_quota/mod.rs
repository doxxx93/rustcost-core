use axum::extract::{State};
use axum::Json;

use crate::api::util::json::to_json;
use crate::api::dto::ApiResponse;
use crate::app_state::AppState;
use crate::errors::AppError;

pub struct InfoK8sResourceQuotaController;

impl InfoK8sResourceQuotaController {
    pub async fn get_k8s_resource_quotas(
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
        to_json(state.info_k8s_service.get_k8s_resource_quotas().await)
    }
}
