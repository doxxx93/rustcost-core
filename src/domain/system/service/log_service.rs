use serde_json::{json, Value};
use crate::core::persistence::logs::log_repository::{LogRepository, LogRepositoryImpl};

pub async fn get_system_log_file_list() -> anyhow::Result<Value> {
    let repo = LogRepositoryImpl::new();
    let file_names = repo.get_system_log_file_list()?;
    Ok(json!({ "fileNames": file_names }))
}

pub async fn get_system_log_lines(
    date: String,
    cursor: usize,
    limit: usize,
) -> anyhow::Result<Value> {
    let repo = LogRepositoryImpl::new();
    let (lines, _next_cursor) = repo
        .get_system_log_lines(&date, cursor, limit)
        .await?;

    Ok(json!({
        "date": date,
        "lines": lines
    }))
}
