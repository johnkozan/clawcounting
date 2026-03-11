use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::user::User;
use crate::services::user_service;

/// Authenticated user, attached to request extensions by the auth middleware.
/// Extract in handlers via `auth: AuthUser`.
#[derive(Clone)]
pub struct AuthUser(pub User);

/// Axum middleware that validates Bearer tokens (JWT or API key).
///
/// - JWT tokens (contain `.`): validated and user loaded by `sub` claim.
/// - API keys (`tsk_...`): SHA-256 hashed and looked up.
///
/// Attaches `AuthUser` to request extensions on success.
pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.strip_prefix("bearer "))
        .ok_or(AppError::Unauthorized)?;

    let token = token.to_string();

    // JWT tokens have exactly two dots (header.payload.signature).
    let user = if token.chars().filter(|c| *c == '.').count() == 2 {
        let jwt_secret = state
            .config
            .jwt_secret
            .as_ref()
            .ok_or(AppError::Internal("JWT secret not configured".into()))?
            .clone();

        let claims = user_service::validate_jwt(&token, &jwt_secret)?;
        if claims.token_type != "access" {
            return Err(AppError::Unauthorized);
        }

        let user_id = claims.sub.clone();
        state
            .with_read(move |conn| {
                user_service::get_user_internal(conn, &user_id).map_err(|_| AppError::Unauthorized)
            })
            .await?
    } else {
        let key_hash = user_service::hash_api_key(&token);
        state
            .with_read(move |conn| user_service::authenticate_by_api_key_hash(conn, &key_hash))
            .await?
    };

    if !user.is_active {
        return Err(AppError::Unauthorized);
    }

    request.extensions_mut().insert(AuthUser(user));
    Ok(next.run(request).await)
}

// ── AuthUser extractor ─────────────────────────────────────────────

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(AppError::Unauthorized)
    }
}
