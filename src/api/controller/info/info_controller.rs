//! Info controller: connects routes to info usecases

use axum::extract::{State};
use axum::Json;
use serde_json::Value;

use crate::api::dto::ApiResponse;
use crate::api::util::json::to_json;
use crate::app_state::AppState;
use crate::core::persistence::info::fixed::unit_price::info_unit_price_entity::InfoUnitPriceEntity;
use crate::core::persistence::info::fixed::version::info_version_entity::InfoVersionEntity;
use crate::domain::info::dto::info_unit_price_upsert_request::InfoUnitPriceUpsertRequest;
use crate::errors::AppError;

pub struct InfoController;

impl InfoController {
    pub async fn get_info_unit_prices(
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<InfoUnitPriceEntity>>, AppError> {
        to_json(state.info_service.get_info_unit_prices().await)
    }

    pub async fn upsert_info_unit_prices(
        State(state): State<AppState>,
        Json(payload): Json<InfoUnitPriceUpsertRequest>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.info_service.upsert_info_unit_prices(payload).await)
    }

    pub async fn get_info_versions(
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<InfoVersionEntity>>, AppError> {
        to_json(state.info_service.get_info_versions().await)
    }
}
