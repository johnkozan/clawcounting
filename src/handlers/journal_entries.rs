use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::journal_entry::{
    CreateJournalEntryRequest, JournalEntry, JournalEntryFilters, JournalEntryWithLines,
    ReverseRequest,
};
use crate::models::{DataResponse, ListResponse};
use crate::services::journal_service;

async fn create_journal_entry(
    State(state): State<AppState>,
    Json(req): Json<CreateJournalEntryRequest>,
) -> Result<Json<DataResponse<JournalEntryWithLines>>, AppError> {
    let user_id = state.system_user_id.clone();
    let entry = state
        .with_write_mut(move |conn| {
            journal_service::create_journal_entry(conn, req, &user_id)
        })
        .await?;
    Ok(Json(DataResponse { data: entry }))
}

async fn list_journal_entries(
    State(state): State<AppState>,
    Query(filters): Query<JournalEntryFilters>,
) -> Result<Json<ListResponse<JournalEntry>>, AppError> {
    let (data, has_more, next_cursor) = state
        .with_read(move |conn| journal_service::list_journal_entries(conn, &filters))
        .await?;
    Ok(Json(ListResponse {
        data,
        has_more,
        next_cursor,
    }))
}

async fn get_journal_entry(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DataResponse<JournalEntryWithLines>>, AppError> {
    let entry = state
        .with_read(move |conn| journal_service::get_journal_entry(conn, &id))
        .await?;
    Ok(Json(DataResponse { data: entry }))
}

async fn reverse_journal_entry(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ReverseRequest>,
) -> Result<Json<DataResponse<JournalEntryWithLines>>, AppError> {
    let user_id = state.system_user_id.clone();
    let entry = state
        .with_write_mut(move |conn| {
            journal_service::reverse_journal_entry(
                conn,
                &id,
                &user_id,
                req.entry_date.as_deref(),
            )
        })
        .await?;
    Ok(Json(DataResponse { data: entry }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_journal_entry).get(list_journal_entries))
        .route("/{id}", get(get_journal_entry))
        .route("/{id}/reverse", post(reverse_journal_entry))
}
