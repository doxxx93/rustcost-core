use axum::extract::{Path, Query, State};
use axum::Json;
use serde_json::Value;

use crate::api::util::json::to_json;
use crate::api::dto::{metrics_dto::RangeQuery, ApiResponse};
use crate::app_state::AppState;
use crate::errors::AppError;

pub struct K8sPodMetricsController;

impl K8sPodMetricsController {
    pub async fn get_metric_k8s_pods_raw(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.metric_service.get_metric_k8s_pods_raw(q).await)
    }

    pub async fn get_metric_k8s_pods_raw_summary(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_pods_raw_summary(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_pods_raw_efficiency(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_pods_raw_efficiency(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_pod_raw(
        State(state): State<AppState>,
        Path(pod_uid): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_pod_raw(pod_uid, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_pod_raw_summary(
        State(state): State<AppState>,
        Path(pod_uid): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_pod_raw_summary(pod_uid, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_pod_raw_efficiency(
        State(state): State<AppState>,
        Path(pod_uid): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_pod_raw_efficiency(pod_uid, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_pods_cost(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(state.metric_service.get_metric_k8s_pods_cost(q).await)
    }

    pub async fn get_metric_k8s_pods_cost_summary(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_pods_cost_summary(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_pods_cost_trend(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_pods_cost_trend(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_pod_cost(
        State(state): State<AppState>,
        Path(pod_uid): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_pod_cost(pod_uid, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_pod_cost_summary(
        State(state): State<AppState>,
        Path(pod_uid): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_pod_cost_summary(pod_uid, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_pod_cost_trend(
        State(state): State<AppState>,
        Path(pod_uid): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_pod_cost_trend(pod_uid, q)
                .await,
        )
    }
}
