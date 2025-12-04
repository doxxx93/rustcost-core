use anyhow::Result;
use chrono::Utc;
use tracing::{debug, error};
use crate::app_state::AppState;

pub async fn run(state: AppState) -> Result<()> {
    let now = Utc::now();
    debug!("Running minutely task (collectors + summarizers)...");

    // Info check (safe and fast)
    let info = super::info::load_info_state().await?;
    debug!("Version: {}", info.version.git_version);
    debug!("Settings: {:?}", info.settings);


    // --- Collectors ---
    if let Err(e) = super::collectors::k8s::run(state, now).await {
        error!(?e, "K8s collector failed");
    }

    if let Err(e) = super::collectors::rustexporter::run(now).await {
        error!(?e, "RustExporter collector failed");
    }

    Ok(())
}

