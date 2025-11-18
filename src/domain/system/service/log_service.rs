use serde_json::{json, Value};
use crate::core::persistence::logs::log_repository::{LogRepository, LogRepositoryImpl};

pub async fn list_logs() -> anyhow::Result<Value> {
    let repo = LogRepositoryImpl::new();
    let logs = repo.get_logs()?;
    Ok(json!({ "logs": logs }))
}

pub async fn show_log_by_date(date: String) -> anyhow::Result<Value> {
    let repo = LogRepositoryImpl::new();
    let lines = repo.get_log(&date)?;
    Ok(json!({ "date": date, "lines": lines }))
}
