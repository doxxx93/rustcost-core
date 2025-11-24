use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde_json::Value;

use crate::api::util::json::to_json;
use crate::api::dto::{metrics_dto::RangeQuery, ApiResponse};
use crate::app_state::AppState;
use crate::errors::AppError;

pub struct K8sNamespaceMetricsController;

impl K8sNamespaceMetricsController {
    pub async fn get_metric_k8s_namespaces_raw(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_namespaces_raw(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_namespaces_raw_summary(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_namespaces_raw_summary(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_namespaces_raw_efficiency(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_namespaces_raw_efficiency(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_namespace_raw(
        State(state): State<AppState>,
        Path(namespace): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_namespace_raw(namespace, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_namespace_raw_summary(
        State(state): State<AppState>,
        Path(namespace): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_namespace_raw_summary(namespace, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_namespace_raw_efficiency(
        State(state): State<AppState>,
        Path(namespace): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_namespace_raw_efficiency(namespace, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_namespaces_cost(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_namespaces_cost(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_namespaces_cost_summary(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_namespaces_cost_summary(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_namespaces_cost_trend(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_namespaces_cost_trend(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_namespace_cost(
        State(state): State<AppState>,
        Path(namespace): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_namespace_cost(namespace, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_namespace_cost_summary(
        State(state): State<AppState>,
        Path(namespace): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_namespace_cost_summary(namespace, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_namespace_cost_trend(
        State(state): State<AppState>,
        Path(namespace): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_namespace_cost_trend(namespace, q)
                .await,
        )
    }
}
