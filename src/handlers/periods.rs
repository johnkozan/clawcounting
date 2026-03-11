use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::period::{CloseQuery, ClosingResult, CreatePeriodRequest, FinancialPeriod};
use crate::models::{DataResponse, DataResponseFinancialPeriod, DataResponseClosingResult, ListResponse, ListResponseFinancialPeriod, PaginationParams};
use crate::services::period_service;

#[utoipa::path(post, path = "/api/v1/periods", request_body = CreatePeriodRequest, responses((status = 200, body = DataResponseFinancialPeriod)), tag = "Periods", security(("bearer" = [])))]
pub async fn create_period(
    State(state): State<AppState>,
    Json(req): Json<CreatePeriodRequest>,
) -> Result<Json<DataResponse<FinancialPeriod>>, AppError> {
    let period = state
        .with_write(|conn| period_service::create_period(conn, req))
        .await?;
    Ok(Json(DataResponse { data: period }))
}

#[utoipa::path(get, path = "/api/v1/periods", params(PaginationParams), responses((status = 200, body = ListResponseFinancialPeriod)), tag = "Periods", security(("bearer" = [])))]
pub async fn list_periods(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<FinancialPeriod>>, AppError> {
    let limit = params.limit();
    let cursor = params.cursor.clone();
    let (data, has_more, next_cursor) = state
        .with_read(move |conn| {
            period_service::list_periods(conn, limit, cursor.as_deref())
        })
        .await?;
    Ok(Json(ListResponse {
        data,
        has_more,
        next_cursor,
    }))
}

#[utoipa::path(get, path = "/api/v1/periods/{id}", responses((status = 200, body = DataResponseFinancialPeriod)), tag = "Periods", security(("bearer" = [])))]
pub async fn get_period(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DataResponse<FinancialPeriod>>, AppError> {
    let period = state
        .with_read(move |conn| period_service::get_period(conn, &id))
        .await?;
    Ok(Json(DataResponse { data: period }))
}

#[utoipa::path(post, path = "/api/v1/periods/{id}/close", params(CloseQuery), responses((status = 200, body = DataResponseClosingResult)), tag = "Periods", security(("bearer" = [])))]
pub async fn close_period(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Query(query): Query<CloseQuery>,
) -> Result<Json<DataResponse<ClosingResult>>, AppError> {
    let user_id = auth.0.id;
    let preview = query.preview;
    let result = state
        .with_write_mut(move |conn| {
            period_service::close_period(conn, &id, &user_id, preview)
        })
        .await?;
    Ok(Json(DataResponse { data: result }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_period).get(list_periods))
        .route("/{id}", get(get_period))
        .route("/{id}/close", post(close_period))
}
