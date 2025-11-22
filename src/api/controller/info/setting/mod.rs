use crate::api::controller::metric::metrics_controller::to_json;
use crate::api::dto::ApiResponse;
use crate::api::util::validation_ext::ValidateRequestExt;
use crate::core::persistence::info::fixed::setting::info_setting_entity::InfoSettingEntity;
use crate::domain::info::dto::info_setting_upsert_request::InfoSettingUpsertRequest;
use axum::Json;
use serde_json::Value;

pub async fn get_info_settings() -> Json<ApiResponse<InfoSettingEntity>> {
    to_json(crate::domain::info::service::info_settings_service::get_info_settings().await)
}

pub async fn upsert_info_settings(
    Json(payload): Json<InfoSettingUpsertRequest>,
) -> Json<ApiResponse<Value>> {
    // Validate first
    let payload = match payload.validate_or_err() {
        Ok(v) => v,
        Err(err_json) => return err_json,
    };

    // Delegate to to_json()
    to_json(crate::domain::info::service::info_settings_service::upsert_info_settings(payload).await)
}
