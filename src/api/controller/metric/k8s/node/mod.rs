use axum::extract::{Path, Query, State};
use axum::Json;
use serde_json::Value;

use crate::api::util::json::to_json;
use crate::api::dto::{metrics_dto::RangeQuery, ApiResponse};
use crate::app_state::AppState;
use crate::errors::AppError;

pub struct K8sNodeMetricsController;

impl K8sNodeMetricsController {
    pub async fn get_metric_k8s_nodes_raw(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.metric_service.get_metric_k8s_nodes_raw(q).await)
    }

    pub async fn get_metric_k8s_nodes_raw_summary(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_nodes_raw_summary(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_nodes_raw_efficiency(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_nodes_raw_efficiency(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_node_raw(
        State(state): State<AppState>,
        Path(node_name): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_node_raw(node_name, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_node_raw_summary(
        State(state): State<AppState>,
        Path(node_name): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_node_raw_summary(node_name, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_node_raw_efficiency(
        State(state): State<AppState>,
        Path(node_name): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_node_raw_efficiency(node_name, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_nodes_cost(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.metric_service.get_metric_k8s_nodes_cost(q).await)
    }

    pub async fn get_metric_k8s_nodes_cost_summary(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_nodes_cost_summary(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_nodes_cost_trend(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_nodes_cost_trend(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_node_cost(
        State(state): State<AppState>,
        Path(node_name): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_node_cost(node_name, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_node_cost_summary(
        State(state): State<AppState>,
        Path(node_name): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_node_cost_summary(node_name, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_node_cost_trend(
        State(state): State<AppState>,
        Path(node_name): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_node_cost_trend(node_name, q)
                .await,
        )
    }
}
