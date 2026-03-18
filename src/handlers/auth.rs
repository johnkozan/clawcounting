use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::user::{
    CreateUserRequest, LoginRequest, RefreshRequest, RefreshResponse, TokenResponse, UserResponse,
};
use crate::models::{DataResponse, DataResponseTokenResponse, DataResponseRefreshResponse, DataResponseUserResponse};
use crate::services::user_service;

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SetupStatusResponse {
    pub needs_setup: bool,
}

#[utoipa::path(post, path = "/auth/login", request_body = LoginRequest, responses((status = 200, body = DataResponseTokenResponse)), tag = "Auth")]
/// POST /auth/login — email + password → JWT token pair
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<DataResponse<TokenResponse>>, AppError> {
    let jwt_secret = state
        .config
        .jwt_secret
        .clone()
        .ok_or(AppError::Internal("JWT secret not configured".into()))?;

    let user = state
        .with_read(move |conn| {
            user_service::authenticate_by_password(conn, &req.email, &req.password)
        })
        .await?;

    let access = user_service::create_access_token(&user.id, &jwt_secret)?;
    let refresh = user_service::create_refresh_token(&user.id, &jwt_secret)?;

    Ok(Json(DataResponse {
        data: TokenResponse {
            access_token: access,
            refresh_token: refresh,
            token_type: "Bearer".to_string(),
            expires_in: user_service::ACCESS_TOKEN_EXPIRY_SECS,
        },
    }))
}

#[utoipa::path(post, path = "/auth/refresh", request_body = RefreshRequest, responses((status = 200, body = DataResponseRefreshResponse)), tag = "Auth")]
/// POST /auth/refresh — refresh token → new access token
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<DataResponse<RefreshResponse>>, AppError> {
    let jwt_secret = state
        .config
        .jwt_secret
        .clone()
        .ok_or(AppError::Internal("JWT secret not configured".into()))?;

    let claims = user_service::validate_jwt(&req.refresh_token, &jwt_secret)?;
    if claims.token_type != "refresh" {
        return Err(AppError::Unauthorized);
    }

    // Verify user still exists and is active
    let user_id = claims.sub.clone();
    let _user = state
        .with_read(move |conn| {
            let u = user_service::get_user_internal(conn, &user_id)?;
            if !u.is_active {
                return Err(AppError::Unauthorized);
            }
            Ok(u)
        })
        .await?;

    let access = user_service::create_access_token(&claims.sub, &jwt_secret)?;

    Ok(Json(DataResponse {
        data: RefreshResponse {
            access_token: access,
            token_type: "Bearer".to_string(),
            expires_in: user_service::ACCESS_TOKEN_EXPIRY_SECS,
        },
    }))
}

#[utoipa::path(get, path = "/auth/me", responses((status = 200, body = DataResponseUserResponse)), tag = "Auth", security(("bearer" = [])))]
/// GET /auth/me — return current user info
pub async fn me(auth: AuthUser) -> Result<Json<DataResponse<UserResponse>>, AppError> {
    let resp: UserResponse = auth.0.into();
    Ok(Json(DataResponse { data: resp }))
}

/// GET /auth/setup/status — check if initial setup is needed
pub async fn setup_status(
    State(state): State<AppState>,
) -> Result<Json<DataResponse<SetupStatusResponse>>, AppError> {
    let has_users = state
        .with_read(|conn| user_service::has_any_users(conn))
        .await?;
    Ok(Json(DataResponse {
        data: SetupStatusResponse {
            needs_setup: !has_users,
        },
    }))
}

/// POST /auth/setup — create the first user (only works when no users exist)
pub async fn setup(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<DataResponse<TokenResponse>>, AppError> {
    let jwt_secret = state
        .config
        .jwt_secret
        .clone()
        .ok_or(AppError::Internal("JWT secret not configured".into()))?;

    let user = state
        .with_write(move |conn| {
            // Check that no users exist — this is the guard
            if user_service::has_any_users(conn)? {
                return Err(AppError::ValidationError {
                    field: "setup".into(),
                    message: "Setup has already been completed".into(),
                    suggestion: "Use /auth/login to sign in".into(),
                });
            }
            user_service::create_user(conn, req)
        })
        .await?;

    let access = user_service::create_access_token(&user.id, &jwt_secret)?;
    let refresh = user_service::create_refresh_token(&user.id, &jwt_secret)?;

    Ok(Json(DataResponse {
        data: TokenResponse {
            access_token: access,
            refresh_token: refresh,
            token_type: "Bearer".to_string(),
            expires_in: user_service::ACCESS_TOKEN_EXPIRY_SECS,
        },
    }))
}

/// Public auth routes (no auth required).
pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/setup", post(setup))
        .route("/setup/status", get(setup_status))
}

/// Protected auth routes (auth required).
pub fn protected_router() -> Router<AppState> {
    Router::new().route("/me", get(me))
}
