use axum::extract::{Path, Query, State};
use axum::Json;
use serde_json::Value;

use crate::api::util::json::to_json;
use crate::api::dto::info_dto::PaginationQuery;
use crate::api::dto::info_dto::K8sListNodeQuery;
use crate::api::dto::ApiResponse;
use crate::api::dto::paginated_response::PaginatedResponse;
use crate::app_state::AppState;
use crate::core::persistence::info::k8s::node::info_node_entity::InfoNodeEntity;
use crate::domain::info::dto::info_k8s_node_patch_request::{
    InfoK8sNodePatchRequest,
    InfoK8sNodePricePatchRequest,
};
use crate::errors::AppError;
use k8s_openapi::api::core::v1::Node;

pub struct InfoK8sNodeController;
pub struct InfoK8sLiveNodeController;

impl InfoK8sNodeController {
    pub async fn get_info_k8s_node(
        State(state): State<AppState>,
        Path(node_name): Path<String>,
    ) -> Result<Json<ApiResponse<InfoNodeEntity>>, AppError> {
        to_json(state.info_k8s_service.get_info_k8s_node(node_name).await)
    }

    pub async fn list_k8s_nodes(
        State(state): State<AppState>,
        Query(filter): Query<K8sListNodeQuery>,
    ) -> Result<Json<ApiResponse<Vec<InfoNodeEntity>>>, AppError> {
        to_json(state.info_k8s_service.list_k8s_nodes(filter).await)
    }

    pub async fn patch_info_k8s_node_filter(
        State(state): State<AppState>,
        Path(id): Path<String>,
        Json(payload): Json<InfoK8sNodePatchRequest>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .patch_info_k8s_node_filter(id, payload)
                .await,
        )
    }

    pub async fn patch_info_k8s_node_price(
        State(state): State<AppState>,
        Path(id): Path<String>,
        Json(payload): Json<InfoK8sNodePricePatchRequest>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .patch_info_k8s_node_price(id, payload)
                .await,
        )
    }
}

impl InfoK8sLiveNodeController {
    pub async fn list_k8s_nodes(
        State(state): State<AppState>,
        Query(pagination): Query<PaginationQuery>,
    ) -> Result<Json<ApiResponse<PaginatedResponse<Node>>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .get_k8s_live_nodes_paginated(pagination.limit, pagination.offset)
                .await,
        )
    }

    pub async fn get_k8s_node(
        Path(node_name): Path<String>,
        State(state): State<AppState>,
    ) -> Result<Json<ApiResponse<Node>>, AppError> {
        to_json(
            state
                .info_k8s_service
                .get_k8s_live_node(node_name)
                .await,
        )
    }
}
