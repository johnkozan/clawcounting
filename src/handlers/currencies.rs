use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::currency::{CreateCurrencyRequest, UpdateCurrencyRequest};
use crate::models::{DataResponse, ListResponse, PaginationParams};
use crate::services::currency_service;

async fn create_currency(
    State(state): State<AppState>,
    Json(req): Json<CreateCurrencyRequest>,
) -> Result<Json<DataResponse<crate::models::currency::Currency>>, AppError> {
    let currency = state
        .with_write(|conn| currency_service::create_currency(conn, req))
        .await?;
    Ok(Json(DataResponse { data: currency }))
}

async fn list_currencies(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListResponse<crate::models::currency::Currency>>, AppError> {
    let limit = params.limit();
    let cursor = params.cursor.clone();
    let (data, has_more, next_cursor) = state
        .with_read(move |conn| {
            currency_service::list_currencies(conn, limit, cursor.as_deref())
        })
        .await?;
    Ok(Json(ListResponse {
        data,
        has_more,
        next_cursor,
    }))
}

async fn get_currency(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DataResponse<crate::models::currency::Currency>>, AppError> {
    let currency = state
        .with_read(move |conn| currency_service::get_currency(conn, &id))
        .await?;
    Ok(Json(DataResponse { data: currency }))
}

async fn update_currency(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCurrencyRequest>,
) -> Result<Json<DataResponse<crate::models::currency::Currency>>, AppError> {
    let currency = state
        .with_write(move |conn| currency_service::update_currency(conn, &id, req))
        .await?;
    Ok(Json(DataResponse { data: currency }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_currency).get(list_currencies))
        .route("/{id}", get(get_currency).patch(update_currency))
}
