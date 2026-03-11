use rusqlite::{Connection, OptionalExtension, params, TransactionBehavior};
use uuid::Uuid;

use crate::db::i128_funcs::{decode_i128, encode_i128};
use crate::error::AppError;
use crate::models::amount::i128_to_decimal_str;
use crate::models::journal_entry::{JournalEntry, JournalEntryLine, JournalEntryWithLines};
use crate::models::period::{ClosingResult, CreatePeriodRequest, FinancialPeriod};

fn row_to_period(row: &rusqlite::Row) -> rusqlite::Result<FinancialPeriod> {
    Ok(FinancialPeriod {
        id: row.get("id")?,
        name: row.get("name")?,
        start_date: row.get("start_date")?,
        end_date: row.get("end_date")?,
        closed_at: row.get("closed_at")?,
        closed_by: row.get("closed_by")?,
        closing_entry_id: row.get("closing_entry_id")?,
        created_at: row.get("created_at")?,
    })
}

pub fn create_period(
    conn: &Connection,
    req: CreatePeriodRequest,
) -> Result<FinancialPeriod, AppError> {
    if req.start_date >= req.end_date {
        return Err(AppError::ValidationError {
            field: "start_date".into(),
            message: "start_date must be before end_date".into(),
            suggestion: "Ensure the period start date is earlier than the end date".into(),
        });
    }

    // Check overlap
    let overlap_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM financial_periods
         WHERE start_date < ?1 AND end_date > ?2",
        params![req.end_date, req.start_date],
        |row| row.get(0),
    )?;
    if overlap_count > 0 {
        return Err(AppError::ValidationError {
            field: "start_date".into(),
            message: "Period overlaps with an existing period".into(),
            suggestion: "Choose dates that don't overlap with existing periods".into(),
        });
    }

    let id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO financial_periods (id, name, start_date, end_date)
         VALUES (?1, ?2, ?3, ?4)",
        params![id, req.name, req.start_date, req.end_date],
    )?;

    get_period(conn, &id)
}

pub fn get_period(conn: &Connection, id: &str) -> Result<FinancialPeriod, AppError> {
    conn.query_row(
        "SELECT id, name, start_date, end_date, closed_at, closed_by, closing_entry_id, created_at
         FROM financial_periods WHERE id = ?1",
        params![id],
        row_to_period,
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound {
        resource: "Financial period".into(),
        id: id.into(),
    })
}

pub fn list_periods(
    conn: &Connection,
    limit: u32,
    cursor: Option<&str>,
) -> Result<(Vec<FinancialPeriod>, bool, Option<String>), AppError> {
    let fetch_limit = limit + 1;

    let mut periods = if let Some(cursor) = cursor {
        let mut stmt = conn.prepare(
            "SELECT id, name, start_date, end_date, closed_at, closed_by, closing_entry_id, created_at
             FROM financial_periods WHERE id > ?1 ORDER BY id LIMIT ?2",
        )?;
        stmt.query_map(params![cursor, fetch_limit], row_to_period)?
            .collect::<Result<Vec<_>, _>>()?
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, name, start_date, end_date, closed_at, closed_by, closing_entry_id, created_at
             FROM financial_periods ORDER BY start_date, id LIMIT ?1",
        )?;
        stmt.query_map(params![fetch_limit], row_to_period)?
            .collect::<Result<Vec<_>, _>>()?
    };

    let has_more = periods.len() > limit as usize;
    if has_more {
        periods.truncate(limit as usize);
    }
    let next_cursor = if has_more {
        periods.last().map(|p| p.id.clone())
    } else {
        None
    };

    Ok((periods, has_more, next_cursor))
}

pub fn find_period_for_date(conn: &Connection, date: &str) -> Result<FinancialPeriod, AppError> {
    conn.query_row(
        "SELECT id, name, start_date, end_date, closed_at, closed_by, closing_entry_id, created_at
         FROM financial_periods
         WHERE start_date <= ?1 AND end_date >= ?1 AND closed_at IS NULL",
        params![date],
        row_to_period,
    )
    .optional()?
    .ok_or_else(|| AppError::ValidationError {
        field: "entry_date".into(),
        message: format!("No open financial period found for date {date}"),
        suggestion: "Create an open period covering this date".into(),
    })
}

pub fn close_period(
    conn: &mut Connection,
    period_id: &str,
    user_id: &str,
    preview: bool,
) -> Result<ClosingResult, AppError> {
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Verify period is open
    let period: FinancialPeriod = tx
        .query_row(
            "SELECT id, name, start_date, end_date, closed_at, closed_by, closing_entry_id, created_at
             FROM financial_periods WHERE id = ?1",
            params![period_id],
            row_to_period,
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound {
            resource: "Financial period".into(),
            id: period_id.into(),
        })?;

    if period.closed_at.is_some() {
        return Err(AppError::PeriodClosed {
            period_id: period_id.into(),
            suggestion: "This period is already closed".into(),
        });
    }

    // Load retained_earnings_account_id
    let re_account_id: String = tx
        .query_row(
            "SELECT value FROM settings WHERE key = 'retained_earnings_account_id'",
            [],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::ValidationError {
            field: "retained_earnings_account_id".into(),
            message: "Retained earnings account is not configured".into(),
            suggestion: "Set retained_earnings_account_id in settings before closing a period"
                .into(),
        })?;

    // Verify RE account exists and is equity type
    let re_type: String = tx.query_row(
        "SELECT account_type FROM accounts WHERE id = ?1",
        params![re_account_id],
        |row| row.get(0),
    ).map_err(|_| AppError::ValidationError {
        field: "retained_earnings_account_id".into(),
        message: "Retained earnings account not found".into(),
        suggestion: "Set a valid equity account as retained_earnings_account_id".into(),
    })?;
    if re_type != "equity" {
        return Err(AppError::ValidationError {
            field: "retained_earnings_account_id".into(),
            message: "Retained earnings account must be an equity account".into(),
            suggestion: "Set an equity-type account as retained_earnings_account_id".into(),
        });
    }

    // Get asset_scale for currency of RE account
    let re_asset_scale: u32 = tx.query_row(
        "SELECT c.asset_scale FROM accounts a JOIN currencies c ON a.currency_id = c.id WHERE a.id = ?1",
        params![re_account_id],
        |row| row.get(0),
    )?;

    // Query revenue/expense balances for this period
    struct AccountBalance {
        account_id: String,
        total_debits: i128,
        total_credits: i128,
        asset_scale: u32,
    }

    let balances: Vec<AccountBalance> = {
        let mut stmt = tx.prepare(
            "SELECT ab.account_id, ab.total_debits, ab.total_credits, c.asset_scale
             FROM account_balances ab
             JOIN accounts a ON ab.account_id = a.id
             JOIN currencies c ON a.currency_id = c.id
             WHERE ab.period_id = ?1 AND a.account_type IN ('revenue', 'expense')",
        )?;
        stmt.query_map(params![period_id], |row| {
            let td: Vec<u8> = row.get("total_debits")?;
            let tc: Vec<u8> = row.get("total_credits")?;
            Ok(AccountBalance {
                account_id: row.get("account_id")?,
                total_debits: decode_i128(&td),
                total_credits: decode_i128(&tc),
                asset_scale: row.get("asset_scale")?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    };

    // Build closing entry lines
    let entry_id = Uuid::now_v7().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let mut lines = Vec::new();
    let mut total_closing_debits: i128 = 0;
    let mut total_closing_credits: i128 = 0;

    for bal in &balances {
        let net_debit = bal.total_debits - bal.total_credits;
        if net_debit == 0 {
            continue;
        }

        let line_id = Uuid::now_v7().to_string();
        let (debit, credit) = if net_debit > 0 {
            // Account has debit balance → credit to zero
            total_closing_credits += net_debit;
            (0i128, net_debit)
        } else {
            // Account has credit balance → debit to zero
            let abs_net = -net_debit;
            total_closing_debits += abs_net;
            (abs_net, 0i128)
        };

        lines.push(JournalEntryLine {
            id: line_id,
            journal_entry_id: entry_id.clone(),
            account_id: bal.account_id.clone(),
            debit_amount: debit.to_string(),
            credit_amount: credit.to_string(),
            display_debit: i128_to_decimal_str(debit, bal.asset_scale),
            display_credit: i128_to_decimal_str(credit, bal.asset_scale),
            description: Some("Period close".into()),
            created_at: now.clone(),
        });
    }

    // Retained earnings line to balance
    let re_difference = total_closing_debits - total_closing_credits;
    if re_difference != 0 {
        let (re_debit, re_credit) = if re_difference > 0 {
            // More debits from closing → credit RE (net income)
            (0i128, re_difference)
        } else {
            // More credits from closing → debit RE (net loss)
            (-re_difference, 0i128)
        };

        lines.push(JournalEntryLine {
            id: Uuid::now_v7().to_string(),
            journal_entry_id: entry_id.clone(),
            account_id: re_account_id.clone(),
            debit_amount: re_debit.to_string(),
            credit_amount: re_credit.to_string(),
            display_debit: i128_to_decimal_str(re_debit, re_asset_scale),
            display_credit: i128_to_decimal_str(re_credit, re_asset_scale),
            description: Some("Net income/loss to retained earnings".into()),
            created_at: now.clone(),
        });
    }

    let entry = JournalEntry {
        id: entry_id.clone(),
        period_id: period_id.to_string(),
        entry_date: period.end_date.clone(),
        posted_at: now.clone(),
        created_by: user_id.to_string(),
        description: format!("Period close: {}", period.name),
        reference: None,
        is_reversal: false,
        reverses_id: None,
        metadata: None,
        created_at: now.clone(),
    };

    let result_entry = JournalEntryWithLines {
        entry: entry.clone(),
        lines: lines.clone(),
    };

    if preview {
        tx.rollback()
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        return Ok(ClosingResult {
            period,
            closing_entry: result_entry,
            preview: true,
        });
    }

    // Insert closing journal entry
    if !lines.is_empty() {
        tx.execute(
            "INSERT INTO journal_entries (id, period_id, entry_date, posted_at, created_by, description, reference, is_reversal, reverses_id, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, 0, NULL, '{}')",
            params![
                entry.id,
                entry.period_id,
                entry.entry_date,
                entry.posted_at,
                entry.created_by,
                entry.description,
            ],
        )?;

        for line in &lines {
            let debit: i128 = line.debit_amount.parse().unwrap_or(0);
            let credit: i128 = line.credit_amount.parse().unwrap_or(0);
            tx.execute(
                "INSERT INTO journal_entry_lines (id, journal_entry_id, account_id, debit_amount, credit_amount, description)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    line.id,
                    line.journal_entry_id,
                    line.account_id,
                    encode_i128(debit).as_slice(),
                    encode_i128(credit).as_slice(),
                    line.description,
                ],
            )?;
        }
    }

    // Close the period
    tx.execute(
        "UPDATE financial_periods SET closed_at = ?1, closed_by = ?2, closing_entry_id = ?3 WHERE id = ?4",
        params![now, user_id, entry_id, period_id],
    )?;

    tx.commit()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Construct updated period (conn is available again after commit)
    let updated_period = FinancialPeriod {
        closed_at: Some(now.clone()),
        closed_by: Some(user_id.to_string()),
        closing_entry_id: Some(entry_id),
        ..period
    };

    Ok(ClosingResult {
        period: updated_period,
        closing_entry: result_entry,
        preview: false,
    })
}
