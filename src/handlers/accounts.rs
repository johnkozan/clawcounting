use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::account::{Account, AccountFilters, CreateAccountRequest, UpdateAccountRequest};
use crate::models::journal_entry::{BalanceQuery, BalanceResponse, JournalEntryLine, TransactionFilters};
use crate::models::{DataResponse, DataResponseAccount, DataResponseBalanceResponse, DataResponseVecAccount, ListResponse, ListResponseAccount, ListResponseJournalEntryLine};
use crate::services::{account_service, balance_service, journal_service};

#[utoipa::path(post, path = "/api/v1/accounts", request_body = CreateAccountRequest, responses((status = 200, body = DataResponseAccount)), tag = "Accounts", security(("bearer" = [])))]
pub async fn create_account(
    State(state): State<AppState>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<DataResponse<Account>>, AppError> {
    let account = state
        .with_write(|conn| account_service::create_account(conn, req))
        .await?;
    Ok(Json(DataResponse { data: account }))
}

#[utoipa::path(get, path = "/api/v1/accounts", params(AccountFilters), responses((status = 200, body = ListResponseAccount)), tag = "Accounts", security(("bearer" = [])))]
pub async fn list_accounts(
    State(state): State<AppState>,
    Query(filters): Query<AccountFilters>,
) -> Result<Json<ListResponse<Account>>, AppError> {
    let (data, has_more, next_cursor) = state
        .with_read(move |conn| account_service::list_accounts(conn, &filters))
        .await?;
    Ok(Json(ListResponse {
        data,
        has_more,
        next_cursor,
    }))
}

#[utoipa::path(get, path = "/api/v1/accounts/{id}", responses((status = 200, body = DataResponseAccount)), tag = "Accounts", security(("bearer" = [])))]
pub async fn get_account(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DataResponse<Account>>, AppError> {
    let account = state
        .with_read(move |conn| account_service::get_account(conn, &id))
        .await?;
    Ok(Json(DataResponse { data: account }))
}

#[utoipa::path(patch, path = "/api/v1/accounts/{id}", request_body = UpdateAccountRequest, responses((status = 200, body = DataResponseAccount)), tag = "Accounts", security(("bearer" = [])))]
pub async fn update_account(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAccountRequest>,
) -> Result<Json<DataResponse<Account>>, AppError> {
    let account = state
        .with_write(move |conn| account_service::update_account(conn, &id, req))
        .await?;
    Ok(Json(DataResponse { data: account }))
}

#[utoipa::path(get, path = "/api/v1/accounts/{id}/sub-accounts", responses((status = 200, body = DataResponseVecAccount)), tag = "Accounts", security(("bearer" = [])))]
pub async fn get_sub_accounts(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DataResponse<Vec<Account>>>, AppError> {
    let subs = state
        .with_read(move |conn| account_service::get_sub_accounts(conn, &id))
        .await?;
    Ok(Json(DataResponse { data: subs }))
}

#[utoipa::path(get, path = "/api/v1/accounts/{id}/balance", params(BalanceQuery), responses((status = 200, body = DataResponseBalanceResponse)), tag = "Accounts", security(("bearer" = [])))]
pub async fn get_balance(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<BalanceQuery>,
) -> Result<Json<DataResponse<BalanceResponse>>, AppError> {
    let balance = state
        .with_read(move |conn| {
            balance_service::get_account_balance(conn, &id, query.period_id.as_deref())
        })
        .await?;
    Ok(Json(DataResponse { data: balance }))
}

#[utoipa::path(get, path = "/api/v1/accounts/{id}/transactions", params(TransactionFilters), responses((status = 200, body = ListResponseJournalEntryLine)), tag = "Accounts", security(("bearer" = [])))]
pub async fn get_transactions(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(filters): Query<TransactionFilters>,
) -> Result<Json<ListResponse<JournalEntryLine>>, AppError> {
    let limit = filters.limit();
    let cursor = filters.cursor.clone();
    let (data, has_more, next_cursor) = state
        .with_read(move |conn| {
            journal_service::get_account_transactions(conn, &id, limit, cursor.as_deref())
        })
        .await?;
    Ok(Json(ListResponse {
        data,
        has_more,
        next_cursor,
    }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_account).get(list_accounts))
        .route("/{id}", get(get_account).patch(update_account))
        .route("/{id}/sub-accounts", get(get_sub_accounts))
        .route("/{id}/balance", get(get_balance))
        .route("/{id}/transactions", get(get_transactions))
}
