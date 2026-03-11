use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::user::{
    CreateServiceAccountRequest, CreateUserRequest, ServiceAccountCreatedResponse,
    UpdateUserRequest, UserResponse,
};
use crate::models::{DataResponse, DataResponseUserResponse, DataResponseServiceAccountCreatedResponse, ListResponse, ListResponseUserResponse, PaginationParams};
use crate::services::user_service;

#[utoipa::path(post, path = "/api/v1/users", request_body = CreateUserRequest, responses((status = 200, body = DataResponseUserResponse)), tag = "Users", security(("bearer" = [])))]
/// POST /api/v1/users — create a human user
pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<DataResponse<UserResponse>>, AppError> {
    let user = state
        .with_write(|conn| user_service::create_user(conn, req))
        .await?;
    Ok(Json(DataResponse { data: user }))
}

#[utoipa::path(get, path = "/api/v1/users", params(PaginationParams), responses((status = 200, body = ListResponseUserResponse)), tag = "Users", security(("bearer" = [])))]
/// GET /api/v1/users — list users
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<UserResponse>>, AppError> {
    let limit = params.limit();
    let cursor = params.cursor.clone();
    let (data, has_more, next_cursor) = state
        .with_read(move |conn| user_service::list_users(conn, limit, cursor.as_deref()))
        .await?;
    Ok(Json(ListResponse {
        data,
        has_more,
        next_cursor,
    }))
}

#[utoipa::path(get, path = "/api/v1/users/{id}", responses((status = 200, body = DataResponseUserResponse)), tag = "Users", security(("bearer" = [])))]
/// GET /api/v1/users/{id} — get single user
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DataResponse<UserResponse>>, AppError> {
    let user = state
        .with_read(move |conn| user_service::get_user(conn, &id))
        .await?;
    Ok(Json(DataResponse { data: user }))
}

#[utoipa::path(patch, path = "/api/v1/users/{id}", request_body = UpdateUserRequest, responses((status = 200, body = DataResponseUserResponse)), tag = "Users", security(("bearer" = [])))]
/// PATCH /api/v1/users/{id} — update user
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<DataResponse<UserResponse>>, AppError> {
    let user = state
        .with_write(move |conn| user_service::update_user(conn, &id, req))
        .await?;
    Ok(Json(DataResponse { data: user }))
}

#[utoipa::path(post, path = "/api/v1/users/service-accounts", request_body = CreateServiceAccountRequest, responses((status = 200, body = DataResponseServiceAccountCreatedResponse)), tag = "Users", security(("bearer" = [])))]
/// POST /api/v1/users/service-accounts — create service account, returns API key once
pub async fn create_service_account(
    State(state): State<AppState>,
    Json(req): Json<CreateServiceAccountRequest>,
) -> Result<Json<DataResponse<ServiceAccountCreatedResponse>>, AppError> {
    let (user, api_key) = state
        .with_write(|conn| user_service::create_service_account(conn, req))
        .await?;
    Ok(Json(DataResponse {
        data: ServiceAccountCreatedResponse { user, api_key },
    }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_user).get(list_users))
        .route("/service-accounts", post(create_service_account))
        .route("/{id}", get(get_user).patch(update_user))
}
