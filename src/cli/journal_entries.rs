use rusqlite::Connection;
use std::path::Path;

use crate::error::AppError;
use crate::models::journal_entry::{CreateJournalEntryRequest, JournalEntryFilters};
use crate::services::journal_service;

pub fn list(
    conn: &Connection,
    period_id: Option<&str>,
    json: bool,
) -> Result<(), AppError> {
    let filters = JournalEntryFilters {
        period_id: period_id.map(String::from),
        start_date: None,
        end_date: None,
        account_id: None,
        limit: Some(200),
        cursor: None,
    };
    let (entries, _, _) = journal_service::list_journal_entries(conn, &filters)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&entries)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
        return Ok(());
    }

    if entries.is_empty() {
        println!("No journal entries found.");
        return Ok(());
    }

    println!(
        "{:<36}  {:<12}  {:<40}  {}",
        "ID", "DATE", "DESCRIPTION", "REV"
    );
    println!("{}", "-".repeat(100));
    for e in &entries {
        let rev = if e.is_reversal { "R" } else { "" };
        let desc = if e.description.len() > 40 {
            format!("{}...", &e.description[..37])
        } else {
            e.description.clone()
        };
        println!("{:<36}  {:<12}  {:<40}  {}", e.id, e.entry_date, desc, rev);
    }
    println!("\n{} entry(ies)", entries.len());
    Ok(())
}

pub fn create_from_file(
    conn: &mut Connection,
    file_path: &str,
    user_id: &str,
    json: bool,
) -> Result<(), AppError> {
    let path = Path::new(file_path);
    let content = std::fs::read_to_string(path).map_err(|e| AppError::Internal(format!("Failed to read file: {e}")))?;
    let req: CreateJournalEntryRequest =
        serde_json::from_str(&content).map_err(|e| AppError::ValidationError {
            field: "file".into(),
            message: format!("Invalid JSON: {e}"),
            suggestion: "Ensure the file contains valid CreateJournalEntryRequest JSON".into(),
        })?;

    let entry = journal_service::create_journal_entry(conn, req, user_id)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&entry)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
    } else {
        println!("Created journal entry: {} [{}]", entry.entry.description, entry.entry.id);
        println!("  Date: {}, Period: {}", entry.entry.entry_date, entry.entry.period_id);
        println!("  Lines: {}", entry.lines.len());
        for line in &entry.lines {
            let dr = if line.debit_amount != "0" {
                format!("DR {}", line.display_debit)
            } else {
                String::new()
            };
            let cr = if line.credit_amount != "0" {
                format!("CR {}", line.display_credit)
            } else {
                String::new()
            };
            println!("    {} {} {}", line.account_id, dr, cr);
        }
    }
    Ok(())
}

pub fn get(conn: &Connection, id: &str, json: bool) -> Result<(), AppError> {
    let entry = journal_service::get_journal_entry(conn, id)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&entry)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
    } else {
        println!("ID:          {}", entry.entry.id);
        println!("Date:        {}", entry.entry.entry_date);
        println!("Description: {}", entry.entry.description);
        println!("Period:      {}", entry.entry.period_id);
        println!("Is Reversal: {}", entry.entry.is_reversal);
        if let Some(ref rev) = entry.entry.reverses_id {
            println!("Reverses:    {}", rev);
        }
        println!("\nLines:");
        println!(
            "  {:<36}  {:<16}  {:<16}  {}",
            "ACCOUNT", "DEBIT", "CREDIT", "DESCRIPTION"
        );
        println!("  {}", "-".repeat(90));
        for line in &entry.lines {
            let dr = if line.debit_amount != "0" {
                &line.display_debit
            } else {
                ""
            };
            let cr = if line.credit_amount != "0" {
                &line.display_credit
            } else {
                ""
            };
            println!(
                "  {:<36}  {:<16}  {:<16}  {}",
                line.account_id,
                dr,
                cr,
                line.description.as_deref().unwrap_or("")
            );
        }
    }
    Ok(())
}

pub fn reverse(
    conn: &mut Connection,
    entry_id: &str,
    user_id: &str,
    date: Option<&str>,
    json: bool,
) -> Result<(), AppError> {
    let entry = journal_service::reverse_journal_entry(conn, entry_id, user_id, date)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&entry)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
    } else {
        println!(
            "Created reversal entry: {} [{}]",
            entry.entry.description, entry.entry.id
        );
        println!("  Reverses: {}", entry.entry.reverses_id.as_deref().unwrap_or(""));
    }
    Ok(())
}
