//! Info API DTOs

use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct K8sPodQueryRequestDto {
    pub start: Option<NaiveDateTime>,
    pub end: Option<NaiveDateTime>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort: Option<String>,
    pub namespace: Option<String>,
    pub node: Option<String>,
    pub deployment: Option<String>,
    pub name: Option<String>,

    /// Filter metrics by the owning team.
    pub team: Option<String>,

    /// Filter metrics by specific microservice name.
    pub service: Option<String>,

    /// Filter by deployment environment.
    /// Common values: `"dev"`, `"stage"`, `"prod"`.
    pub env: Option<String>,

}