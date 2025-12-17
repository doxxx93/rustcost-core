//! Info API DTOs

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct K8sListQuery {
    pub namespace: Option<String>,
    #[serde(alias = "label-selector")]
    pub label_selector: Option<String>,
    pub node_name: Option<String>, // for pods by node
}

#[derive(Deserialize, Debug, Default)]
pub struct K8sListNodeQuery {
    #[serde(alias = "label-selector")]
    pub label_selector: Option<String>,
    pub team: Option<String>,
    pub service: Option<String>,
    pub env: Option<String>, // "dev", "stage", "prod"
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct PaginationQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
