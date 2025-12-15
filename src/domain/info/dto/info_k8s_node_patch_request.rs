
use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::core::persistence::info::k8s::node::info_node_entity::NodePricePeriod;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InfoK8sNodePatchRequest {
    // --- Team / Service metadata (NEW) ---
    pub team: Option<String>,
    pub service: Option<String>,
    pub env: Option<String>, // "dev", "stage", "prod"
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InfoK8sNodePricePatchRequest {
    /// Fixed price for this node in USD (instance / VM / bare metal)
    pub fixed_instance_usd: Option<f64>,

    /// Billing period for `fixed_instance`
    pub price_period: Option<NodePricePeriod>,
}
