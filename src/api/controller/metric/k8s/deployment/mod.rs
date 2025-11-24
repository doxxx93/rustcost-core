use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde_json::Value;

use crate::api::util::json::to_json;
use crate::api::dto::{metrics_dto::RangeQuery, ApiResponse};
use crate::app_state::AppState;
use crate::errors::AppError;

pub struct K8sDeploymentMetricsController;

impl K8sDeploymentMetricsController {
    pub async fn get_metric_k8s_deployments_raw(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_deployments_raw(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_deployments_raw_summary(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_deployments_raw_summary(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_deployments_raw_efficiency(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_deployments_raw_efficiency(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_deployment_raw(
        State(state): State<AppState>,
        Path(deployment): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_deployment_raw(deployment, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_deployment_raw_summary(
        State(state): State<AppState>,
        Path(deployment): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_deployment_raw_summary(deployment, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_deployment_raw_efficiency(
        State(state): State<AppState>,
        Path(deployment): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_deployment_raw_efficiency(deployment, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_deployments_cost(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_deployments_cost(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_deployments_cost_summary(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_deployments_cost_summary(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_deployments_cost_trend(
        State(state): State<AppState>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_deployments_cost_trend(q)
                .await,
        )
    }

    pub async fn get_metric_k8s_deployment_cost(
        State(state): State<AppState>,
        Path(deployment): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_deployment_cost(deployment, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_deployment_cost_summary(
        State(state): State<AppState>,
        Path(deployment): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_deployment_cost_summary(deployment, q)
                .await,
        )
    }

    pub async fn get_metric_k8s_deployment_cost_trend(
        State(state): State<AppState>,
        Path(deployment): Path<String>,
        Query(q): Query<RangeQuery>,
    ) -> Result<Json<ApiResponse<Value>>, AppError> {
        to_json(
            state
                .metric_service
                .get_metric_k8s_deployment_cost_trend(deployment, q)
                .await,
        )
    }
}
