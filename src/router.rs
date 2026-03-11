use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::app_state::AppState;

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        // Placeholder route groups for future endpoints
        .nest("/api/v1/currencies", Router::new())
        .nest("/api/v1/accounts", Router::new())
        .nest("/api/v1/journal-entries", Router::new())
        .nest("/api/v1/periods", Router::new())
        .nest("/api/v1/reports", Router::new())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
