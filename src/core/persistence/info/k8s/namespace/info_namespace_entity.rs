use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InfoNamespaceEntity {
    pub name: Option<String>,
    pub uid: Option<String>,
    // TODO: Add fields needed for cost tracking
}
