use axum::extract::State;
use axum::routing::patch;
use axum::{Json, Router};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::DataResponse;
use crate::services::settings_service;

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub retained_earnings_account_id: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct SettingsResponse {
    pub retained_earnings_account_id: Option<String>,
}

async fn update_settings(
    State(state): State<AppState>,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<DataResponse<SettingsResponse>>, AppError> {
    let settings = state
        .with_write(move |conn| {
            if let Some(ref id) = req.retained_earnings_account_id {
                // Validate account exists and is equity type
                let acct_type: String = conn
                    .query_row(
                        "SELECT account_type FROM accounts WHERE id = ?1",
                        rusqlite::params![id],
                        |row| row.get(0),
                    )
                    .map_err(|_| AppError::ValidationError {
                        field: "retained_earnings_account_id".into(),
                        message: "Account not found".into(),
                        suggestion: "Provide a valid account ID".into(),
                    })?;
                if acct_type != "equity" {
                    return Err(AppError::ValidationError {
                        field: "retained_earnings_account_id".into(),
                        message: "Account must be an equity account".into(),
                        suggestion: "Select an equity-type account for retained earnings".into(),
                    });
                }
                settings_service::set_setting(conn, "retained_earnings_account_id", id)?;
            }

            let re_id = settings_service::get_retained_earnings_account_id(conn)?;
            Ok(SettingsResponse {
                retained_earnings_account_id: re_id,
            })
        })
        .await?;
    Ok(Json(DataResponse { data: settings }))
}

async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<DataResponse<SettingsResponse>>, AppError> {
    let settings = state
        .with_read(move |conn| {
            let re_id = settings_service::get_retained_earnings_account_id(conn)?;
            Ok(SettingsResponse {
                retained_earnings_account_id: re_id,
            })
        })
        .await?;
    Ok(Json(DataResponse { data: settings }))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", patch(update_settings).get(get_settings))
}
