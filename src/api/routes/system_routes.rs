//! System routes (e.g., /api/v1/system/*)

use axum::{routing::{get, post}, Router};
use crate::api::controller::system as sc;

pub fn system_routes() -> Router {
    Router::new()
        .route("/status", get(sc::status))
        .route("/health", get(sc::health))
        .route("/backup", post(sc::backup))
        .route("/resync", post(sc::resync))

        .route("/log-files", get(sc::get_system_log_file_list))
        .route("/log-files/{date}", get(sc::get_system_log_lines))
}
