use rusqlite::Connection;

use crate::error::AppError;
use crate::models::period::CreatePeriodRequest;
use crate::services::period_service;

pub fn list(conn: &Connection, json: bool) -> Result<(), AppError> {
    let (periods, _, _) = period_service::list_periods(conn, 200, None)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&periods)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
        return Ok(());
    }

    if periods.is_empty() {
        println!("No financial periods found.");
        return Ok(());
    }

    println!(
        "{:<36}  {:<20}  {:<12}  {:<12}  {}",
        "ID", "NAME", "START", "END", "STATUS"
    );
    println!("{}", "-".repeat(100));
    for p in &periods {
        let status = if p.closed_at.is_some() {
            "closed"
        } else {
            "open"
        };
        println!(
            "{:<36}  {:<20}  {:<12}  {:<12}  {}",
            p.id, p.name, p.start_date, p.end_date, status
        );
    }
    println!("\n{} period(s)", periods.len());
    Ok(())
}

pub fn create(
    conn: &Connection,
    name: &str,
    start_date: &str,
    end_date: &str,
    json: bool,
) -> Result<(), AppError> {
    let req = CreatePeriodRequest {
        name: name.to_string(),
        start_date: start_date.to_string(),
        end_date: end_date.to_string(),
    };
    let period = period_service::create_period(conn, req)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&period)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
    } else {
        println!(
            "Created period: {} ({} to {}) [{}]",
            period.name, period.start_date, period.end_date, period.id
        );
    }
    Ok(())
}

pub fn close(
    conn: &mut Connection,
    period_id: &str,
    user_id: &str,
    preview: bool,
    json: bool,
) -> Result<(), AppError> {
    let result = period_service::close_period(conn, period_id, user_id, preview)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&result)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
    } else if preview {
        println!("=== PREVIEW: Closing Entry for {} ===", result.period.name);
        println!("Description: {}", result.closing_entry.entry.description);
        println!("Lines:");
        for line in &result.closing_entry.lines {
            let dr = if line.display_debit != "0.00" && line.debit_amount != "0" {
                &line.display_debit
            } else {
                ""
            };
            let cr = if line.display_credit != "0.00" && line.credit_amount != "0" {
                &line.display_credit
            } else {
                ""
            };
            println!(
                "  {} | DR: {:<16} CR: {:<16} | {}",
                line.account_id,
                dr,
                cr,
                line.description.as_deref().unwrap_or("")
            );
        }
        println!("\n(Preview only — no changes committed)");
    } else {
        println!("Period '{}' closed successfully.", result.period.name);
        println!("Closing entry: {}", result.closing_entry.entry.id);
    }
    Ok(())
}
