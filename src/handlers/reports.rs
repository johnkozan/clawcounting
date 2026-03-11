use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::report::*;
use crate::models::DataResponse;
use crate::services::report_service;

async fn trial_balance(
    State(state): State<AppState>,
    Query(query): Query<TrialBalanceQuery>,
) -> Result<Json<DataResponse<TrialBalanceReport>>, AppError> {
    let report = state
        .with_read(move |conn| report_service::trial_balance(conn, &query))
        .await?;
    Ok(Json(DataResponse { data: report }))
}

async fn balance_sheet(
    State(state): State<AppState>,
    Query(query): Query<BalanceSheetQuery>,
) -> Result<Json<DataResponse<BalanceSheetReport>>, AppError> {
    let report = state
        .with_read(move |conn| report_service::balance_sheet(conn, &query))
        .await?;
    Ok(Json(DataResponse { data: report }))
}

async fn income_statement(
    State(state): State<AppState>,
    Query(query): Query<IncomeStatementQuery>,
) -> Result<Json<DataResponse<IncomeStatementReport>>, AppError> {
    let report = state
        .with_read(move |conn| report_service::income_statement(conn, &query))
        .await?;
    Ok(Json(DataResponse { data: report }))
}

async fn general_ledger(
    State(state): State<AppState>,
    Query(query): Query<GeneralLedgerQuery>,
) -> Result<Json<DataResponse<GeneralLedgerReport>>, AppError> {
    let report = state
        .with_read(move |conn| report_service::general_ledger(conn, &query))
        .await?;
    Ok(Json(DataResponse { data: report }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/trial-balance", get(trial_balance))
        .route("/balance-sheet", get(balance_sheet))
        .route("/income-statement", get(income_statement))
        .route("/general-ledger", get(general_ledger))
}
