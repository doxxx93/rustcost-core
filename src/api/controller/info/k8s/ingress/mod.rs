use axum::extract::{Path, Query, State};
use axum::Json;
use k8s_openapi::api::networking::v1::Ingress;

use crate::api::dto::ApiResponse;
use crate::api::dto::info_dto::PaginationQuery;
use crate::api::dto::paginated_response::PaginatedResponse;
use crate::api::util::json::to_json;
use crate::app_state::AppState;
use crate::errors::AppError;

pub struct InfoK8sIngressController;

impl InfoK8sIngressController {
    pub async fn get_k8s_ingresses(
        State(state): State<AppState>,
        Query(pagination): Query<PaginationQuery>,
    ) -> Result<Json<ApiResponse<PaginatedResponse<Ingress>>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .get_k8s_ingresses_paginated(pagination.limit, pagination.offset)
                .await,
        )
    }

    pub async fn get_k8s_ingress(
        Path((namespace, name)): Path<(String, String)>,
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<Ingress>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .get_k8s_ingress(namespace, name)
                .await,
        )
    }
}
