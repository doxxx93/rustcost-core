use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InfoDeploymentEntity {
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub replicas: Option<i32>,
    // TODO: Add fields needed for cost tracking
}
