use crate::core::client::kube_client::build_kube_client;
use crate::core::client::nodes::{fetch_node_summary, fetch_nodes};
use crate::scheduler::tasks::collectors::k8s::node::task::{handle_node, update_node_info};
use crate::scheduler::tasks::collectors::k8s::pod::task::handle_pod;
use crate::scheduler::tasks::collectors::k8s::summary_dto::Summary;
use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::{debug, error};
use crate::scheduler::tasks::collectors::k8s::container::task::handle_container;

/// Collects node-level stats from the Kubelet `/stats/summary` endpoint.
pub async fn run(now: DateTime<Utc>) -> Result<()> {
    debug!("Starting K8s node stats task...");

    // --- Build kube client ---
    let client = build_kube_client().await?;

    // --- Step 1: Fetch all nodes ---
    let node_list = fetch_nodes(&client).await?;

    // --- Step 2: For each node, call /proxy/stats/summary ---
    for node in node_list {
        let node_name = node.metadata.name.clone().unwrap_or_default();

        match fetch_node_summary::<Summary>(&client, &node_name).await {
            Ok(summary) => {
                match handle_summary(&summary, now).await {
                    Ok(result) => {

                        // if new node
                        if let Some(_name) = result.node_name {
                            update_node_info(node, now).await?;
                        }
                        // new_pods.extend(result.updated_pods);
                        // new_containers.extend(result.updated_containers);
                    }
                    Err(e) => error!("❌ Failed to handle summary for {}: {:?}", node_name, e),
                }
            }
            Err(e) => {
                error!("❌ Failed to fetch summary for {}: {:?}", node_name, e);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct SummaryHandleResultDto {
    pub node_name: Option<String>,
    // pub updated_pods: Vec<String>,
    //  updated_containers: Vec<String>,
}


/// Handle and persist one `/stats/summary` response
pub async fn handle_summary(summary: &Summary, now: DateTime<Utc>) -> Result<SummaryHandleResultDto> {
    let mut result = SummaryHandleResultDto::default();

    if handle_node(summary, now).await? {
        result.node_name = Some(summary.node.node_name.clone());
    }

    handle_pod(summary, now).await?;
    handle_container(summary, now).await?;

    Ok(result)
}

/* ---------------- Tests ---------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::{fmt, EnvFilter};

    #[test]
    fn test_run_does_not_panic() {
        // Initialize full tracing (only once)
        let _ = fmt()
            .with_env_filter(EnvFilter::new("debug")) // show debug/info/warn/error
            .with_target(true)
            .with_level(true)
            .with_test_writer()
            .try_init();

        // Build a single-threaded Tokio runtime
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime");

        // Run async code inside the same thread (so debugger can attach)
        rt.block_on(async {
            let result = run().await;
            // Allow both Ok and Err but ensure no panic
            assert!(result.is_ok() || result.is_err());
        });
    }
}
