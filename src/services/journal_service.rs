use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use uuid::Uuid;

use crate::db::i128_funcs::{decode_i128, encode_i128};
use crate::error::AppError;
use crate::models::amount::{decimal_str_to_i128, i128_to_decimal_str};
use crate::models::journal_entry::*;
use crate::services::account_service;
use crate::services::period_service;

fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<JournalEntry> {
    let metadata_str: Option<String> = row.get("metadata")?;
    Ok(JournalEntry {
        id: row.get("id")?,
        period_id: row.get("period_id")?,
        entry_date: row.get("entry_date")?,
        posted_at: row.get("posted_at")?,
        created_by: row.get("created_by")?,
        description: row.get("description")?,
        reference: row.get("reference")?,
        is_reversal: row.get::<_, i32>("is_reversal")? != 0,
        reverses_id: row.get("reverses_id")?,
        metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
        created_at: row.get("created_at")?,
    })
}

fn load_lines(
    conn: &Connection,
    entry_id: &str,
) -> Result<Vec<JournalEntryLine>, AppError> {
    // We need asset_scale to format display amounts
    let mut stmt = conn.prepare(
        "SELECT jel.id, jel.journal_entry_id, jel.account_id,
                jel.debit_amount, jel.credit_amount, jel.description, jel.created_at,
                c.asset_scale
         FROM journal_entry_lines jel
         JOIN accounts a ON jel.account_id = a.id
         JOIN currencies c ON a.currency_id = c.id
         WHERE jel.journal_entry_id = ?1
         ORDER BY jel.id",
    )?;

    let lines = stmt
        .query_map(params![entry_id], |row| {
            let debit_blob: Vec<u8> = row.get("debit_amount")?;
            let credit_blob: Vec<u8> = row.get("credit_amount")?;
            let asset_scale: u32 = row.get("asset_scale")?;
            let debit = decode_i128(&debit_blob);
            let credit = decode_i128(&credit_blob);

            Ok(JournalEntryLine {
                id: row.get("id")?,
                journal_entry_id: row.get("journal_entry_id")?,
                account_id: row.get("account_id")?,
                debit_amount: debit.to_string(),
                credit_amount: credit.to_string(),
                display_debit: i128_to_decimal_str(debit, asset_scale),
                display_credit: i128_to_decimal_str(credit, asset_scale),
                description: row.get("description")?,
                created_at: row.get("created_at")?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(lines)
}

pub fn create_journal_entry(
    conn: &mut Connection,
    req: CreateJournalEntryRequest,
    user_id: &str,
) -> Result<JournalEntryWithLines, AppError> {
    if req.lines.len() < 2 {
        return Err(AppError::ValidationError {
            field: "lines".into(),
            message: "Journal entry must have at least 2 lines".into(),
            suggestion: "Add at least one debit and one credit line".into(),
        });
    }

    // Validate all accounts and parse amounts
    struct ParsedLine {
        account_id: String,
        debit: i128,
        credit: i128,
        description: Option<String>,
        asset_scale: u32,
    }

    let mut parsed_lines = Vec::new();
    let mut currency_id: Option<String> = None;
    let mut has_debit = false;
    let mut has_credit = false;

    for (i, line) in req.lines.iter().enumerate() {
        let account = account_service::get_account(conn, &line.account_id)?;

        if !account.is_active {
            return Err(AppError::ValidationError {
                field: format!("lines[{i}].account_id"),
                message: format!("Account '{}' is inactive", account.name),
                suggestion: "Use an active account".into(),
            });
        }
        if account.has_subledger {
            return Err(AppError::ValidationError {
                field: format!("lines[{i}].account_id"),
                message: format!(
                    "Cannot post directly to control account '{}'. Post to sub-accounts instead.",
                    account.name
                ),
                suggestion: "Use a sub-account of this control account".into(),
            });
        }

        // Verify all accounts use same currency
        match &currency_id {
            None => currency_id = Some(account.currency_id.clone()),
            Some(cid) => {
                if *cid != account.currency_id {
                    return Err(AppError::ValidationError {
                        field: format!("lines[{i}].account_id"),
                        message: "All accounts in a journal entry must use the same currency"
                            .into(),
                        suggestion: "Ensure all line items reference accounts with the same currency".into(),
                    });
                }
            }
        }

        // Get asset scale for amount parsing
        let asset_scale: u32 = conn.query_row(
            "SELECT asset_scale FROM currencies WHERE id = ?1",
            params![account.currency_id],
            |row| row.get(0),
        )?;

        let debit = match &line.debit_amount {
            Some(s) if !s.is_empty() && s != "0" => {
                decimal_str_to_i128(s, asset_scale).map_err(|msg| AppError::ValidationError {
                    field: format!("lines[{i}].debit_amount"),
                    message: msg,
                    suggestion: "Provide a valid decimal amount".into(),
                })?
            }
            _ => 0i128,
        };
        let credit = match &line.credit_amount {
            Some(s) if !s.is_empty() && s != "0" => {
                decimal_str_to_i128(s, asset_scale).map_err(|msg| AppError::ValidationError {
                    field: format!("lines[{i}].credit_amount"),
                    message: msg,
                    suggestion: "Provide a valid decimal amount".into(),
                })?
            }
            _ => 0i128,
        };

        if debit < 0 || credit < 0 {
            return Err(AppError::ValidationError {
                field: format!("lines[{i}]"),
                message: "Amounts must be non-negative".into(),
                suggestion: "Use positive amounts for debits and credits".into(),
            });
        }
        if debit > 0 && credit > 0 {
            return Err(AppError::ValidationError {
                field: format!("lines[{i}]"),
                message: "A line cannot have both debit and credit amounts".into(),
                suggestion: "Each line should have either a debit or a credit, not both".into(),
            });
        }
        if debit == 0 && credit == 0 {
            return Err(AppError::ValidationError {
                field: format!("lines[{i}]"),
                message: "A line must have either a debit or credit amount".into(),
                suggestion: "Provide a non-zero debit or credit amount".into(),
            });
        }

        if debit > 0 {
            has_debit = true;
        }
        if credit > 0 {
            has_credit = true;
        }

        parsed_lines.push(ParsedLine {
            account_id: line.account_id.clone(),
            debit,
            credit,
            description: line.description.clone(),
            asset_scale,
        });
    }

    if !has_debit || !has_credit {
        return Err(AppError::ValidationError {
            field: "lines".into(),
            message: "Entry must have at least one debit line and one credit line".into(),
            suggestion: "Add both debit and credit lines".into(),
        });
    }

    // Verify balanced
    let total_debits: i128 = parsed_lines.iter().map(|l| l.debit).sum();
    let total_credits: i128 = parsed_lines.iter().map(|l| l.credit).sum();
    if total_debits != total_credits {
        let scale = parsed_lines[0].asset_scale;
        return Err(AppError::Unbalanced {
            total_debits: i128_to_decimal_str(total_debits, scale),
            total_credits: i128_to_decimal_str(total_credits, scale),
        });
    }

    // Find open period
    let period = period_service::find_period_for_date(conn, &req.entry_date)?;

    // Begin transaction
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Re-verify period is still open inside transaction
    let still_open: bool = tx
        .query_row(
            "SELECT closed_at IS NULL FROM financial_periods WHERE id = ?1",
            params![period.id],
            |row| row.get(0),
        )
        .unwrap_or(false);
    if !still_open {
        return Err(AppError::PeriodClosed {
            period_id: period.id,
            suggestion: "The period was closed between validation and commit".into(),
        });
    }

    let entry_id = Uuid::now_v7().to_string();
    let metadata_json = req
        .metadata
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap_or_else(|_| "{}".to_string()));

    tx.execute(
        "INSERT INTO journal_entries (id, period_id, entry_date, created_by, description, reference, is_reversal, reverses_id, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, NULL, ?7)",
        params![
            entry_id,
            period.id,
            req.entry_date,
            user_id,
            req.description,
            req.reference,
            metadata_json,
        ],
    )?;

    for pl in &parsed_lines {
        let line_id = Uuid::now_v7().to_string();
        tx.execute(
            "INSERT INTO journal_entry_lines (id, journal_entry_id, account_id, debit_amount, credit_amount, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                line_id,
                entry_id,
                pl.account_id,
                encode_i128(pl.debit).as_slice(),
                encode_i128(pl.credit).as_slice(),
                pl.description,
            ],
        )?;
    }

    tx.commit()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    get_journal_entry(conn, &entry_id)
}

pub fn get_journal_entry(
    conn: &Connection,
    id: &str,
) -> Result<JournalEntryWithLines, AppError> {
    let entry = conn
        .query_row(
            "SELECT id, period_id, entry_date, posted_at, created_by, description, reference,
                    is_reversal, reverses_id, metadata, created_at
             FROM journal_entries WHERE id = ?1",
            params![id],
            row_to_entry,
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound {
            resource: "Journal entry".into(),
            id: id.into(),
        })?;

    let lines = load_lines(conn, id)?;

    Ok(JournalEntryWithLines { entry, lines })
}

pub fn list_journal_entries(
    conn: &Connection,
    filters: &JournalEntryFilters,
) -> Result<(Vec<JournalEntry>, bool, Option<String>), AppError> {
    let limit = filters.limit();
    let fetch_limit = limit + 1;

    let mut conditions = vec!["1=1".to_string()];
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref pid) = filters.period_id {
        param_values.push(Box::new(pid.clone()));
        conditions.push(format!("je.period_id = ?{}", param_values.len()));
    }
    if let Some(ref sd) = filters.start_date {
        param_values.push(Box::new(sd.clone()));
        conditions.push(format!("je.entry_date >= ?{}", param_values.len()));
    }
    if let Some(ref ed) = filters.end_date {
        param_values.push(Box::new(ed.clone()));
        conditions.push(format!("je.entry_date <= ?{}", param_values.len()));
    }
    if let Some(ref aid) = filters.account_id {
        param_values.push(Box::new(aid.clone()));
        conditions.push(format!(
            "je.id IN (SELECT journal_entry_id FROM journal_entry_lines WHERE account_id = ?{})",
            param_values.len()
        ));
    }
    if let Some(ref cursor) = filters.cursor {
        param_values.push(Box::new(cursor.clone()));
        conditions.push(format!("je.id > ?{}", param_values.len()));
    }

    param_values.push(Box::new(fetch_limit));
    let limit_idx = param_values.len();

    let sql = format!(
        "SELECT je.id, je.period_id, je.entry_date, je.posted_at, je.created_by,
                je.description, je.reference, je.is_reversal, je.reverses_id,
                je.metadata, je.created_at
         FROM journal_entries je
         WHERE {} ORDER BY je.id LIMIT ?{}",
        conditions.join(" AND "),
        limit_idx
    );

    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let mut entries = stmt
        .query_map(params_refs.as_slice(), row_to_entry)?
        .collect::<Result<Vec<_>, _>>()?;

    let has_more = entries.len() > limit as usize;
    if has_more {
        entries.truncate(limit as usize);
    }
    let next_cursor = if has_more {
        entries.last().map(|e| e.id.clone())
    } else {
        None
    };

    Ok((entries, has_more, next_cursor))
}

pub fn get_account_transactions(
    conn: &Connection,
    account_id: &str,
    limit: u32,
    cursor: Option<&str>,
) -> Result<(Vec<JournalEntryLine>, bool, Option<String>), AppError> {
    let fetch_limit = limit + 1;

    let mut lines = if let Some(cursor) = cursor {
        let mut stmt = conn.prepare(
            "SELECT jel.id, jel.journal_entry_id, jel.account_id,
                    jel.debit_amount, jel.credit_amount, jel.description, jel.created_at,
                    c.asset_scale
             FROM journal_entry_lines jel
             JOIN accounts a ON jel.account_id = a.id
             JOIN currencies c ON a.currency_id = c.id
             WHERE jel.account_id = ?1 AND jel.id > ?2
             ORDER BY jel.id LIMIT ?3",
        )?;
        stmt.query_map(params![account_id, cursor, fetch_limit], |row| {
            let debit_blob: Vec<u8> = row.get("debit_amount")?;
            let credit_blob: Vec<u8> = row.get("credit_amount")?;
            let asset_scale: u32 = row.get("asset_scale")?;
            let debit = decode_i128(&debit_blob);
            let credit = decode_i128(&credit_blob);
            Ok(JournalEntryLine {
                id: row.get("id")?,
                journal_entry_id: row.get("journal_entry_id")?,
                account_id: row.get("account_id")?,
                debit_amount: debit.to_string(),
                credit_amount: credit.to_string(),
                display_debit: i128_to_decimal_str(debit, asset_scale),
                display_credit: i128_to_decimal_str(credit, asset_scale),
                description: row.get("description")?,
                created_at: row.get("created_at")?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    } else {
        let mut stmt = conn.prepare(
            "SELECT jel.id, jel.journal_entry_id, jel.account_id,
                    jel.debit_amount, jel.credit_amount, jel.description, jel.created_at,
                    c.asset_scale
             FROM journal_entry_lines jel
             JOIN accounts a ON jel.account_id = a.id
             JOIN currencies c ON a.currency_id = c.id
             WHERE jel.account_id = ?1
             ORDER BY jel.id LIMIT ?2",
        )?;
        stmt.query_map(params![account_id, fetch_limit], |row| {
            let debit_blob: Vec<u8> = row.get("debit_amount")?;
            let credit_blob: Vec<u8> = row.get("credit_amount")?;
            let asset_scale: u32 = row.get("asset_scale")?;
            let debit = decode_i128(&debit_blob);
            let credit = decode_i128(&credit_blob);
            Ok(JournalEntryLine {
                id: row.get("id")?,
                journal_entry_id: row.get("journal_entry_id")?,
                account_id: row.get("account_id")?,
                debit_amount: debit.to_string(),
                credit_amount: credit.to_string(),
                display_debit: i128_to_decimal_str(debit, asset_scale),
                display_credit: i128_to_decimal_str(credit, asset_scale),
                description: row.get("description")?,
                created_at: row.get("created_at")?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    };

    let has_more = lines.len() > limit as usize;
    if has_more {
        lines.truncate(limit as usize);
    }
    let next_cursor = if has_more {
        lines.last().map(|l| l.id.clone())
    } else {
        None
    };

    Ok((lines, has_more, next_cursor))
}

pub fn reverse_journal_entry(
    conn: &mut Connection,
    entry_id: &str,
    user_id: &str,
    entry_date: Option<&str>,
) -> Result<JournalEntryWithLines, AppError> {
    // Load original entry
    let original = get_journal_entry(conn, entry_id)?;

    if original.entry.is_reversal {
        return Err(AppError::ValidationError {
            field: "entry_id".into(),
            message: "Cannot reverse a reversal entry".into(),
            suggestion: "Only original entries can be reversed".into(),
        });
    }

    // Check if already reversed
    let already_reversed: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM journal_entries WHERE reverses_id = ?1",
            params![entry_id],
            |row| row.get(0),
        )
        .unwrap_or(false);
    if already_reversed {
        return Err(AppError::ValidationError {
            field: "entry_id".into(),
            message: "This entry has already been reversed".into(),
            suggestion: "An entry can only be reversed once".into(),
        });
    }

    let reversal_date = entry_date.unwrap_or(&original.entry.entry_date);

    // Find open period for reversal date
    let period = period_service::find_period_for_date(conn, reversal_date)?;

    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Re-verify period is open
    let still_open: bool = tx
        .query_row(
            "SELECT closed_at IS NULL FROM financial_periods WHERE id = ?1",
            params![period.id],
            |row| row.get(0),
        )
        .unwrap_or(false);
    if !still_open {
        return Err(AppError::PeriodClosed {
            period_id: period.id,
            suggestion: "The period was closed".into(),
        });
    }

    let new_entry_id = Uuid::now_v7().to_string();
    let metadata_json = original
        .entry
        .metadata
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap_or_else(|_| "{}".to_string()));

    tx.execute(
        "INSERT INTO journal_entries (id, period_id, entry_date, created_by, description, reference, is_reversal, reverses_id, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8)",
        params![
            new_entry_id,
            period.id,
            reversal_date,
            user_id,
            format!("Reversal of: {}", original.entry.description),
            original.entry.reference,
            entry_id,
            metadata_json,
        ],
    )?;

    // Swap debits and credits from original lines
    for line in &original.lines {
        let line_id = Uuid::now_v7().to_string();
        let orig_debit: i128 = line.debit_amount.parse().unwrap_or(0);
        let orig_credit: i128 = line.credit_amount.parse().unwrap_or(0);

        tx.execute(
            "INSERT INTO journal_entry_lines (id, journal_entry_id, account_id, debit_amount, credit_amount, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                line_id,
                new_entry_id,
                line.account_id,
                encode_i128(orig_credit).as_slice(), // swapped
                encode_i128(orig_debit).as_slice(),  // swapped
                line.description,
            ],
        )?;
    }

    tx.commit()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    get_journal_entry(conn, &new_entry_id)
}
