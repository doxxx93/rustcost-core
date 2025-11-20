//! System API DTOs
use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
pub struct LogQuery {
    pub cursor: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct PaginatedLogResponse {
    pub date: String,
    pub lines: Vec<String>,
    pub next_cursor: Option<usize>,
}