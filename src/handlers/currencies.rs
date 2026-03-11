use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::currency::{CreateCurrencyRequest, UpdateCurrencyRequest};
use crate::models::{DataResponse, DataResponseCurrency, ListResponse, ListResponseCurrency, PaginationParams};
use crate::services::currency_service;

#[utoipa::path(post, path = "/api/v1/currencies", request_body = CreateCurrencyRequest, responses((status = 200, body = DataResponseCurrency)), tag = "Currencies", security(("bearer" = [])))]
pub async fn create_currency(
    State(state): State<AppState>,
    Json(req): Json<CreateCurrencyRequest>,
) -> Result<Json<DataResponse<crate::models::currency::Currency>>, AppError> {
    let currency = state
        .with_write(|conn| currency_service::create_currency(conn, req))
        .await?;
    Ok(Json(DataResponse { data: currency }))
}

#[utoipa::path(get, path = "/api/v1/currencies", params(PaginationParams), responses((status = 200, body = ListResponseCurrency)), tag = "Currencies", security(("bearer" = [])))]
pub async fn list_currencies(
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

#[utoipa::path(get, path = "/api/v1/currencies/{id}", responses((status = 200, body = DataResponseCurrency)), tag = "Currencies", security(("bearer" = [])))]
pub async fn get_currency(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DataResponse<crate::models::currency::Currency>>, AppError> {
    let currency = state
        .with_read(move |conn| currency_service::get_currency(conn, &id))
        .await?;
    Ok(Json(DataResponse { data: currency }))
}

#[utoipa::path(patch, path = "/api/v1/currencies/{id}", request_body = UpdateCurrencyRequest, responses((status = 200, body = DataResponseCurrency)), tag = "Currencies", security(("bearer" = [])))]
pub async fn update_currency(
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
