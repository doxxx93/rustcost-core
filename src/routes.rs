use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use crate::app_state::AppState;

/// Build the main application router
pub fn app_router() -> Router<AppState> {
    // Metrics, Info, System subrouters live under /api/v1
    let api_v1 = Router::new()
        .nest("/metrics", crate::api::routes::metrics_routes::metrics_routes())
        .nest("/info", crate::api::routes::info_routes::info_routes())
        .nest("/system", crate::api::routes::system_routes::system_routes())
        .nest("/llm", crate::api::routes::llm_routes::llm_routes())
        .nest("/states", crate::api::routes::state_routes::state_routes());

    Router::new()
        // Root route
        .route("/", get(root))
        // Health check
        .route("/health", get(health_check))
        // API v1
        .nest("/api/v1", api_v1)

        // Fallback handler for 404
        .fallback(handler_404)
        // Attach shared application state ONCE here
        // ✅ Apply CORS layer to all routes
        .layer(CorsLayer::very_permissive())
}

// Handler for root
async fn root() -> &'static str {
    "Server is running!"
}

// Handler for health check
async fn health_check() -> &'static str {
    "OK"
}

// Handler for 404 Not Found
async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "The requested resource was not found",
    )
}
