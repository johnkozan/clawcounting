use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::app_state::AppState;
use crate::handlers;
use crate::middleware::auth::require_auth;
use crate::openapi::ApiDoc;

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

pub fn build_router(state: AppState) -> Router {
    let auth_middleware =
        axum::middleware::from_fn_with_state(state.clone(), require_auth);

    // All /api/v1/* routes require authentication
    let api_v1 = Router::new()
        .nest("/currencies", handlers::currencies::router())
        .nest("/accounts", handlers::accounts::router())
        .nest(
            "/journal-entries",
            handlers::journal_entries::router(),
        )
        .nest("/periods", handlers::periods::router())
        .nest("/reports", handlers::reports::router())
        .nest("/settings", handlers::settings::router())
        .nest("/users", handlers::users::router())
        .route_layer(auth_middleware.clone());

    // /auth/me requires auth; /auth/login and /auth/refresh do not
    let auth_routes = handlers::auth::public_router()
        .merge(handlers::auth::protected_router().route_layer(auth_middleware));

    // Static frontend serving (SPA fallback)
    let frontend_dir = std::env::var("CLAWCOUNTING_FRONTEND_DIR")
        .unwrap_or_else(|_| "./frontend/build".to_string());
    let index_file = format!("{}/index.html", frontend_dir);
    let serve_frontend = ServeDir::new(&frontend_dir)
        .not_found_service(ServeFile::new(&index_file));

    Router::new()
        .route("/health", get(health))
        .nest("/auth", auth_routes)
        .nest("/api/v1", api_v1)
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api/v1/openapi.json", ApiDoc::openapi()),
        )
        .fallback_service(serve_frontend)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
