use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {resource} with id {id}")]
    NotFound { resource: String, id: String },

    #[error("Validation error on {field}: {message}")]
    ValidationError {
        field: String,
        message: String,
        suggestion: String,
    },

    #[error("Period {period_id} is closed")]
    PeriodClosed {
        period_id: String,
        suggestion: String,
    },

    #[error("Journal entry is unbalanced: debits={total_debits}, credits={total_credits}")]
    Unbalanced {
        total_debits: String,
        total_credits: String,
    },

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggestion: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match &self {
            AppError::NotFound { resource, id } => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    code: "NOT_FOUND",
                    message: format!("{resource} with id '{id}' not found"),
                    field: None,
                    suggestion: None,
                },
            ),
            AppError::ValidationError {
                field,
                message,
                suggestion,
            } => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    code: "VALIDATION_ERROR",
                    message: message.clone(),
                    field: Some(field.clone()),
                    suggestion: Some(suggestion.clone()),
                },
            ),
            AppError::PeriodClosed {
                period_id,
                suggestion,
            } => (
                StatusCode::CONFLICT,
                ErrorResponse {
                    code: "PERIOD_CLOSED",
                    message: format!("Period {period_id} is closed"),
                    field: None,
                    suggestion: Some(suggestion.clone()),
                },
            ),
            AppError::Unbalanced {
                total_debits,
                total_credits,
            } => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    code: "UNBALANCED_ENTRY",
                    message: format!(
                        "Journal entry is unbalanced: total debits ({total_debits}) != total credits ({total_credits})"
                    ),
                    field: None,
                    suggestion: Some(
                        "Ensure the sum of all debit amounts equals the sum of all credit amounts"
                            .to_string(),
                    ),
                },
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    code: "UNAUTHORIZED",
                    message: "Authentication required".to_string(),
                    field: None,
                    suggestion: Some(
                        "Provide a valid API key or JWT token in the Authorization header"
                            .to_string(),
                    ),
                },
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                ErrorResponse {
                    code: "FORBIDDEN",
                    message: "Insufficient permissions".to_string(),
                    field: None,
                    suggestion: None,
                },
            ),
            AppError::DatabaseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    code: "DATABASE_ERROR",
                    message: msg.clone(),
                    field: None,
                    suggestion: None,
                },
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    code: "INTERNAL_ERROR",
                    message: msg.clone(),
                    field: None,
                    suggestion: None,
                },
            ),
        };

        (status, axum::Json(body)).into_response()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}
