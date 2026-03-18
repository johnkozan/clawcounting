use axum::body::Body;
use axum::http::{HeaderValue, StatusCode, Uri, header};
use axum::response::{IntoResponse, Json, Response};
use axum::{Router, routing::get};
use rust_embed::Embed;
use serde_json::{Value, json};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::app_state::AppState;
use crate::handlers;
use crate::middleware::auth::require_auth;
use crate::openapi::ApiDoc;

#[derive(Embed)]
#[folder = "frontend/build"]
struct FrontendAssets;

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn serve_frontend(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    // Try the exact path first, then fall back to index.html (SPA routing)
    let file = FrontendAssets::get(path)
        .or_else(|| FrontendAssets::get("index.html"));

    match file {
        Some(content) => {
            let mime = if path.is_empty() || !path.contains('.') {
                "text/html".to_string()
            } else {
                mime_guess::from_path(path)
                    .first_or_octet_stream()
                    .to_string()
            };

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, HeaderValue::from_str(&mime).unwrap())
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
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

    Router::new()
        .route("/health", get(health))
        .nest("/auth", auth_routes)
        .nest("/api/v1", api_v1)
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api/v1/openapi.json", ApiDoc::openapi()),
        )
        .fallback(serve_frontend)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
