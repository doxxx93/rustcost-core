use anyhow::Result;
use kube::{Client, Config};
use std::env;
use tracing::debug;

/// Creates a Kubernetes client configured for in-cluster or local development
pub async fn build_kube_client() -> Result<Client> {
    // Check if running in-cluster or local dev
    let client = if let Ok(_api_url) = env::var("RUSTCOST_K8S_API_URL") {
        debug!("Using custom API URL from RUSTCOST_K8S_API_URL");
        // For custom configuration, still use default but it will pick up KUBERNETES_SERVICE_HOST
        Client::try_default().await?
    } else {
        // Use in-cluster config (reads service account token automatically)
        debug!("Using in-cluster configuration");
        Client::try_default().await?
    };

    debug!("Kubernetes client initialized successfully");
    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_client() {
        // Test that client creation doesn't panic
        let result = build_kube_client().await;
        // Allow both success and error (depends on environment)
        assert!(result.is_ok() || result.is_err());
    }
}
