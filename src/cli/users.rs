use rusqlite::Connection;

use crate::error::AppError;
use crate::models::user::{CreateServiceAccountRequest, CreateUserRequest};
use crate::services::user_service;

pub fn list(conn: &Connection, json: bool) -> Result<(), AppError> {
    let (users, _has_more, _next_cursor) = user_service::list_users(conn, 200, None)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&users).unwrap());
        return Ok(());
    }

    println!(
        "{:<38} {:<20} {:<30} {:<8}",
        "ID", "NAME", "EMAIL", "ACTIVE"
    );
    println!("{}", "-".repeat(98));
    for u in &users {
        println!(
            "{:<38} {:<20} {:<30} {:<8}",
            u.id,
            u.name,
            u.email.as_deref().unwrap_or("-"),
            if u.is_active { "yes" } else { "no" }
        );
    }
    println!("\n{} user(s)", users.len());
    Ok(())
}

pub fn create(
    conn: &Connection,
    name: &str,
    email: &str,
    password: &str,
    json_output: bool,
) -> Result<(), AppError> {
    let req = CreateUserRequest {
        name: name.to_string(),
        email: email.to_string(),
        password: password.to_string(),
        permissions: serde_json::json!({}),
    };
    let user = user_service::create_user(conn, req)?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&user).unwrap());
    } else {
        println!("Created user: {} ({})", user.name, user.id);
    }
    Ok(())
}

pub fn create_service_account(
    conn: &Connection,
    name: &str,
    permissions: &str,
    json_output: bool,
) -> Result<(), AppError> {
    let perms: serde_json::Value = serde_json::from_str(permissions).map_err(|e| {
        AppError::ValidationError {
            field: "permissions".into(),
            message: format!("Invalid JSON: {e}"),
            suggestion: "Provide valid JSON, e.g. '{\"journals:create\":true}'".into(),
        }
    })?;

    let req = CreateServiceAccountRequest {
        name: name.to_string(),
        permissions: perms,
    };
    let (user, api_key) = user_service::create_service_account(conn, req)?;

    if json_output {
        let resp = serde_json::json!({
            "user": user,
            "api_key": api_key,
        });
        println!("{}", serde_json::to_string_pretty(&resp).unwrap());
    } else {
        println!("Created service account: {} ({})", user.name, user.id);
        println!("\nAPI Key (save this — it will not be shown again):");
        println!("  {api_key}");
    }
    Ok(())
}
