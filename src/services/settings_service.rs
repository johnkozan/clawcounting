use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::error::AppError;

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    let val = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?;
    Ok(val)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_retained_earnings_account_id(conn: &Connection) -> Result<Option<String>, AppError> {
    get_setting(conn, "retained_earnings_account_id")
}

/// Ensure a system user exists for CLI and unauthenticated operations.
/// Returns the system user's ID.
pub fn ensure_system_user(conn: &Connection) -> Result<String, AppError> {
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM users WHERE email = 'system@clawcounting.local'",
            [],
            |row| row.get(0),
        )
        .optional()?;

    if let Some(id) = existing {
        return Ok(id);
    }

    let id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO users (id, name, email, password_hash, permissions)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            id,
            "System",
            "system@clawcounting.local",
            "!system-no-login",
            r#"{"admin":true}"#
        ],
    )?;
    Ok(id)
}
