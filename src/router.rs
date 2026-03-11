use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::app_state::AppState;
use crate::handlers;

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/api/v1/currencies", handlers::currencies::router())
        .nest("/api/v1/accounts", handlers::accounts::router())
        .nest("/api/v1/journal-entries", handlers::journal_entries::router())
        .nest("/api/v1/periods", handlers::periods::router())
        .nest("/api/v1/settings", handlers::settings::router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
