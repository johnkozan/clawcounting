use rusqlite::{Connection, OptionalExtension, params};

use crate::db::i128_funcs::decode_i128;
use crate::error::AppError;
use crate::models::amount::i128_to_decimal_str;
use crate::models::report::*;
use crate::services::{account_service, period_service};

fn compute_net(total_debits: i128, total_credits: i128, normal_balance: &str) -> i128 {
    if normal_balance == "debit" {
        total_debits - total_credits
    } else {
        total_credits - total_debits
    }
}

fn decode_opt(blob: Option<Vec<u8>>) -> i128 {
    blob.map(|b| decode_i128(&b)).unwrap_or(0)
}

// ── Trial Balance ──────────────────────────────────────────────

struct RawBalanceRow {
    account_id: String,
    account_number: String,
    account_name: String,
    account_type: String,
    normal_balance: String,
    total_debits: i128,
    total_credits: i128,
    asset_scale: u32,
}

pub fn trial_balance(
    conn: &Connection,
    query: &TrialBalanceQuery,
) -> Result<TrialBalanceReport, AppError> {
    let rows = if let Some(ref period_id) = query.period_id {
        // Validate period exists
        period_service::get_period(conn, period_id)?;
        fetch_period_balances(conn, Some(period_id), query.currency_id.as_deref(), None)?
    } else {
        fetch_alltime_balances(conn, query.currency_id.as_deref(), None)?
    };

    let default_scale = rows.first().map(|r| r.asset_scale).unwrap_or(2);
    let mut grand_debits: i128 = 0;
    let mut grand_credits: i128 = 0;

    let result_rows: Vec<TrialBalanceRow> = rows
        .iter()
        .map(|r| {
            grand_debits += r.total_debits;
            grand_credits += r.total_credits;
            TrialBalanceRow {
                account_id: r.account_id.clone(),
                account_number: r.account_number.clone(),
                account_name: r.account_name.clone(),
                account_type: r.account_type.clone(),
                debit_total: r.total_debits.to_string(),
                credit_total: r.total_credits.to_string(),
                display_debit_total: i128_to_decimal_str(r.total_debits, r.asset_scale),
                display_credit_total: i128_to_decimal_str(r.total_credits, r.asset_scale),
            }
        })
        .collect();

    Ok(TrialBalanceReport {
        period_id: query.period_id.clone(),
        currency_id: query.currency_id.clone(),
        rows: result_rows,
        grand_total_debits: grand_debits.to_string(),
        grand_total_credits: grand_credits.to_string(),
        display_grand_total_debits: i128_to_decimal_str(grand_debits, default_scale),
        display_grand_total_credits: i128_to_decimal_str(grand_credits, default_scale),
        is_balanced: grand_debits == grand_credits,
    })
}

/// Fetch balances for a specific period from account_balances.
fn fetch_period_balances(
    conn: &Connection,
    period_id: Option<&str>,
    currency_id: Option<&str>,
    account_type_filter: Option<&[&str]>,
) -> Result<Vec<RawBalanceRow>, AppError> {
    let mut conditions = Vec::new();
    let mut param_vals: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(pid) = period_id {
        param_vals.push(Box::new(pid.to_string()));
        conditions.push(format!("ab.period_id = ?{}", param_vals.len()));
    }
    if let Some(cid) = currency_id {
        param_vals.push(Box::new(cid.to_string()));
        conditions.push(format!("c.id = ?{}", param_vals.len()));
    }
    if let Some(types) = account_type_filter {
        let placeholders: Vec<String> = types
            .iter()
            .map(|t| {
                param_vals.push(Box::new(t.to_string()));
                format!("?{}", param_vals.len())
            })
            .collect();
        conditions.push(format!(
            "a.account_type IN ({})",
            placeholders.join(", ")
        ));
    }

    let where_clause = if conditions.is_empty() {
        "1=1".to_string()
    } else {
        conditions.join(" AND ")
    };

    let sql = format!(
        "SELECT a.id, a.account_number, a.name, a.account_type, a.normal_balance,
                ab.total_debits, ab.total_credits, c.asset_scale
         FROM account_balances ab
         JOIN accounts a ON ab.account_id = a.id
         JOIN currencies c ON a.currency_id = c.id
         WHERE {where_clause}
         ORDER BY a.account_number"
    );

    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_vals.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params_refs.as_slice(), |row| {
            let td_blob: Vec<u8> = row.get("total_debits")?;
            let tc_blob: Vec<u8> = row.get("total_credits")?;
            Ok(RawBalanceRow {
                account_id: row.get("id")?,
                account_number: row.get("account_number")?,
                account_name: row.get("name")?,
                account_type: row.get("account_type")?,
                normal_balance: row.get("normal_balance")?,
                total_debits: decode_i128(&td_blob),
                total_credits: decode_i128(&tc_blob),
                asset_scale: row.get("asset_scale")?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Fetch balances summed across all periods from account_balances.
fn fetch_alltime_balances(
    conn: &Connection,
    currency_id: Option<&str>,
    account_type_filter: Option<&[&str]>,
) -> Result<Vec<RawBalanceRow>, AppError> {
    let mut conditions = Vec::new();
    let mut param_vals: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(cid) = currency_id {
        param_vals.push(Box::new(cid.to_string()));
        conditions.push(format!("c.id = ?{}", param_vals.len()));
    }
    if let Some(types) = account_type_filter {
        let placeholders: Vec<String> = types
            .iter()
            .map(|t| {
                param_vals.push(Box::new(t.to_string()));
                format!("?{}", param_vals.len())
            })
            .collect();
        conditions.push(format!(
            "a.account_type IN ({})",
            placeholders.join(", ")
        ));
    }

    let where_clause = if conditions.is_empty() {
        "1=1".to_string()
    } else {
        conditions.join(" AND ")
    };

    let sql = format!(
        "SELECT a.id, a.account_number, a.name, a.account_type, a.normal_balance,
                sum_i128(ab.total_debits) as total_debits,
                sum_i128(ab.total_credits) as total_credits,
                c.asset_scale
         FROM account_balances ab
         JOIN accounts a ON ab.account_id = a.id
         JOIN currencies c ON a.currency_id = c.id
         WHERE {where_clause}
         GROUP BY ab.account_id
         ORDER BY a.account_number"
    );

    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_vals.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params_refs.as_slice(), |row| {
            let td_blob: Option<Vec<u8>> = row.get("total_debits")?;
            let tc_blob: Option<Vec<u8>> = row.get("total_credits")?;
            Ok(RawBalanceRow {
                account_id: row.get("id")?,
                account_number: row.get("account_number")?,
                account_name: row.get("name")?,
                account_type: row.get("account_type")?,
                normal_balance: row.get("normal_balance")?,
                total_debits: decode_opt(td_blob),
                total_credits: decode_opt(tc_blob),
                asset_scale: row.get("asset_scale")?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Fetch cumulative balances through a period (sum all periods with end_date <= target).
fn fetch_cumulative_through_period(
    conn: &Connection,
    period_id: &str,
    account_type_filter: Option<&[&str]>,
) -> Result<Vec<RawBalanceRow>, AppError> {
    let target_end: String = conn.query_row(
        "SELECT end_date FROM financial_periods WHERE id = ?1",
        params![period_id],
        |row| row.get(0),
    )?;

    let mut conditions = vec!["fp.end_date <= ?1".to_string()];
    let mut param_vals: Vec<Box<dyn rusqlite::types::ToSql>> =
        vec![Box::new(target_end)];

    if let Some(types) = account_type_filter {
        let placeholders: Vec<String> = types
            .iter()
            .map(|t| {
                param_vals.push(Box::new(t.to_string()));
                format!("?{}", param_vals.len())
            })
            .collect();
        conditions.push(format!(
            "a.account_type IN ({})",
            placeholders.join(", ")
        ));
    }

    let sql = format!(
        "SELECT a.id, a.account_number, a.name, a.account_type, a.normal_balance,
                sum_i128(ab.total_debits) as total_debits,
                sum_i128(ab.total_credits) as total_credits,
                c.asset_scale
         FROM account_balances ab
         JOIN accounts a ON ab.account_id = a.id
         JOIN currencies c ON a.currency_id = c.id
         JOIN financial_periods fp ON ab.period_id = fp.id
         WHERE {}
         GROUP BY ab.account_id
         ORDER BY a.account_number",
        conditions.join(" AND ")
    );

    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_vals.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params_refs.as_slice(), |row| {
            let td_blob: Option<Vec<u8>> = row.get("total_debits")?;
            let tc_blob: Option<Vec<u8>> = row.get("total_credits")?;
            Ok(RawBalanceRow {
                account_id: row.get("id")?,
                account_number: row.get("account_number")?,
                account_name: row.get("name")?,
                account_type: row.get("account_type")?,
                normal_balance: row.get("normal_balance")?,
                total_debits: decode_opt(td_blob),
                total_credits: decode_opt(tc_blob),
                asset_scale: row.get("asset_scale")?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Fetch cumulative balances from journal_entry_lines up to a date.
fn fetch_balances_as_of_date(
    conn: &Connection,
    as_of_date: &str,
    account_type_filter: Option<&[&str]>,
) -> Result<Vec<RawBalanceRow>, AppError> {
    let mut conditions = vec!["je.entry_date <= ?1".to_string()];
    let mut param_vals: Vec<Box<dyn rusqlite::types::ToSql>> =
        vec![Box::new(as_of_date.to_string())];

    if let Some(types) = account_type_filter {
        let placeholders: Vec<String> = types
            .iter()
            .map(|t| {
                param_vals.push(Box::new(t.to_string()));
                format!("?{}", param_vals.len())
            })
            .collect();
        conditions.push(format!(
            "a.account_type IN ({})",
            placeholders.join(", ")
        ));
    }

    let sql = format!(
        "SELECT a.id, a.account_number, a.name, a.account_type, a.normal_balance,
                sum_i128(jel.debit_amount) as total_debits,
                sum_i128(jel.credit_amount) as total_credits,
                c.asset_scale
         FROM journal_entry_lines jel
         JOIN journal_entries je ON jel.journal_entry_id = je.id
         JOIN accounts a ON jel.account_id = a.id
         JOIN currencies c ON a.currency_id = c.id
         WHERE {}
         GROUP BY jel.account_id
         ORDER BY a.account_number",
        conditions.join(" AND ")
    );

    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_vals.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params_refs.as_slice(), |row| {
            let td_blob: Option<Vec<u8>> = row.get("total_debits")?;
            let tc_blob: Option<Vec<u8>> = row.get("total_credits")?;
            Ok(RawBalanceRow {
                account_id: row.get("id")?,
                account_number: row.get("account_number")?,
                account_name: row.get("name")?,
                account_type: row.get("account_type")?,
                normal_balance: row.get("normal_balance")?,
                total_debits: decode_opt(td_blob),
                total_credits: decode_opt(tc_blob),
                asset_scale: row.get("asset_scale")?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

// ── Balance Sheet ──────────────────────────────────────────────

pub fn balance_sheet(
    conn: &Connection,
    query: &BalanceSheetQuery,
) -> Result<BalanceSheetReport, AppError> {
    let bs_types: &[&str] = &["asset", "liability", "equity"];

    let rows = if let Some(ref period_id) = query.period_id {
        period_service::get_period(conn, period_id)?;
        fetch_cumulative_through_period(conn, period_id, Some(bs_types))?
    } else if let Some(ref as_of_date) = query.as_of_date {
        fetch_balances_as_of_date(conn, as_of_date, Some(bs_types))?
    } else {
        // Current: sum all periods
        fetch_alltime_balances(conn, None, Some(bs_types))?
    };

    let default_scale = rows.first().map(|r| r.asset_scale).unwrap_or(2);

    let mut assets = Vec::new();
    let mut liabilities = Vec::new();
    let mut equity = Vec::new();
    let mut total_assets: i128 = 0;
    let mut total_liabilities: i128 = 0;
    let mut total_equity: i128 = 0;

    for r in &rows {
        let net = compute_net(r.total_debits, r.total_credits, &r.normal_balance);
        let row = BalanceSheetRow {
            account_id: r.account_id.clone(),
            account_number: r.account_number.clone(),
            account_name: r.account_name.clone(),
            net_balance: net.to_string(),
            display_balance: i128_to_decimal_str(net, r.asset_scale),
        };
        match r.account_type.as_str() {
            "asset" => {
                total_assets += net;
                assets.push(row);
            }
            "liability" => {
                total_liabilities += net;
                liabilities.push(row);
            }
            "equity" => {
                total_equity += net;
                equity.push(row);
            }
            _ => {}
        }
    }

    let total_l_and_e = total_liabilities + total_equity;

    Ok(BalanceSheetReport {
        as_of_date: query.as_of_date.clone(),
        period_id: query.period_id.clone(),
        assets: BalanceSheetSection {
            label: "Assets".to_string(),
            accounts: assets,
            total: total_assets.to_string(),
            display_total: i128_to_decimal_str(total_assets, default_scale),
        },
        liabilities: BalanceSheetSection {
            label: "Liabilities".to_string(),
            accounts: liabilities,
            total: total_liabilities.to_string(),
            display_total: i128_to_decimal_str(total_liabilities, default_scale),
        },
        equity: BalanceSheetSection {
            label: "Equity".to_string(),
            accounts: equity,
            total: total_equity.to_string(),
            display_total: i128_to_decimal_str(total_equity, default_scale),
        },
        total_assets: total_assets.to_string(),
        total_liabilities_and_equity: total_l_and_e.to_string(),
        display_total_assets: i128_to_decimal_str(total_assets, default_scale),
        display_total_liabilities_and_equity: i128_to_decimal_str(total_l_and_e, default_scale),
        is_balanced: total_assets == total_l_and_e,
    })
}

// ── Income Statement ───────────────────────────────────────────

pub fn income_statement(
    conn: &Connection,
    query: &IncomeStatementQuery,
) -> Result<IncomeStatementReport, AppError> {
    let period_id = query.period_id.as_deref().ok_or_else(|| AppError::ValidationError {
        field: "period_id".into(),
        message: "period_id is required for income statement".into(),
        suggestion: "Provide a period_id query parameter".into(),
    })?;

    period_service::get_period(conn, period_id)?;

    let is_types: &[&str] = &["revenue", "expense"];
    let rows = fetch_period_balances(conn, Some(period_id), None, Some(is_types))?;

    let default_scale = rows.first().map(|r| r.asset_scale).unwrap_or(2);
    let mut revenue_rows = Vec::new();
    let mut expense_rows = Vec::new();
    let mut total_revenue: i128 = 0;
    let mut total_expenses: i128 = 0;

    for r in &rows {
        let net = compute_net(r.total_debits, r.total_credits, &r.normal_balance);
        let row = IncomeStatementRow {
            account_id: r.account_id.clone(),
            account_number: r.account_number.clone(),
            account_name: r.account_name.clone(),
            net_amount: net.to_string(),
            display_amount: i128_to_decimal_str(net, r.asset_scale),
        };
        match r.account_type.as_str() {
            "revenue" => {
                total_revenue += net;
                revenue_rows.push(row);
            }
            "expense" => {
                total_expenses += net;
                expense_rows.push(row);
            }
            _ => {}
        }
    }

    let net_income = total_revenue - total_expenses;

    Ok(IncomeStatementReport {
        period_id: period_id.to_string(),
        revenue: revenue_rows,
        expenses: expense_rows,
        total_revenue: total_revenue.to_string(),
        total_expenses: total_expenses.to_string(),
        net_income: net_income.to_string(),
        display_total_revenue: i128_to_decimal_str(total_revenue, default_scale),
        display_total_expenses: i128_to_decimal_str(total_expenses, default_scale),
        display_net_income: i128_to_decimal_str(net_income, default_scale),
    })
}

// ── General Ledger ─────────────────────────────────────────────

struct GlCursor {
    line_id: String,
    entry_date: String,
    posted_at: String,
}

struct RawGlLine {
    id: String,
    journal_entry_id: String,
    entry_date: String,
    description: String,
    reference: Option<String>,
    debit: i128,
    credit: i128,
    asset_scale: u32,
}

pub fn general_ledger(
    conn: &Connection,
    query: &GeneralLedgerQuery,
) -> Result<GeneralLedgerReport, AppError> {
    let account_id = query.account_id.as_deref().ok_or_else(|| AppError::ValidationError {
        field: "account_id".into(),
        message: "account_id is required for general ledger".into(),
        suggestion: "Provide an account_id query parameter".into(),
    })?;

    let account = account_service::get_account(conn, account_id)?;

    let asset_scale: u32 = conn.query_row(
        "SELECT asset_scale FROM currencies WHERE id = ?1",
        params![account.currency_id],
        |row| row.get(0),
    )?;

    // Resolve date range from period_id if provided
    let (range_start, range_end) = if let Some(ref period_id) = query.period_id {
        let period = period_service::get_period(conn, period_id)?;
        (Some(period.start_date), Some(period.end_date))
    } else {
        (query.start_date.clone(), query.end_date.clone())
    };

    let ascending = query.is_ascending();
    let limit = query.limit();

    // Resolve cursor if present
    let cursor = if let Some(ref cursor_id) = query.cursor {
        let row: Option<(String, String)> = conn
            .query_row(
                "SELECT je.entry_date, je.posted_at
                 FROM journal_entry_lines jel
                 JOIN journal_entries je ON jel.journal_entry_id = je.id
                 WHERE jel.id = ?1",
                params![cursor_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        match row {
            Some((entry_date, posted_at)) => Some(GlCursor {
                line_id: cursor_id.clone(),
                entry_date,
                posted_at,
            }),
            None => {
                return Err(AppError::ValidationError {
                    field: "cursor".into(),
                    message: "Invalid cursor: line not found".into(),
                    suggestion: "Use a valid cursor from a previous response".into(),
                });
            }
        }
    } else {
        None
    };

    // Compute starting balance
    let starting_net = compute_gl_starting_balance(
        conn,
        account_id,
        &account.normal_balance,
        ascending,
        range_start.as_deref(),
        range_end.as_deref(),
        cursor.as_ref(),
    )?;

    // Fetch lines
    let mut raw_lines = fetch_gl_lines(
        conn,
        account_id,
        ascending,
        range_start.as_deref(),
        range_end.as_deref(),
        cursor.as_ref(),
        limit,
    )?;

    let has_more = raw_lines.len() > limit as usize;
    if has_more {
        raw_lines.truncate(limit as usize);
    }

    // Compute running balances
    let mut running = starting_net;
    let mut result_lines = Vec::with_capacity(raw_lines.len());

    for line in &raw_lines {
        let net_effect = compute_net(line.debit, line.credit, &account.normal_balance);

        if ascending {
            running += net_effect;
            result_lines.push(GeneralLedgerLine {
                line_id: line.id.clone(),
                journal_entry_id: line.journal_entry_id.clone(),
                entry_date: line.entry_date.clone(),
                description: line.description.clone(),
                reference: line.reference.clone(),
                debit_amount: line.debit.to_string(),
                credit_amount: line.credit.to_string(),
                display_debit: i128_to_decimal_str(line.debit, line.asset_scale),
                display_credit: i128_to_decimal_str(line.credit, line.asset_scale),
                running_balance: running.to_string(),
                display_running_balance: i128_to_decimal_str(running, line.asset_scale),
            });
        } else {
            // Descending: show balance AFTER this transaction, then subtract
            result_lines.push(GeneralLedgerLine {
                line_id: line.id.clone(),
                journal_entry_id: line.journal_entry_id.clone(),
                entry_date: line.entry_date.clone(),
                description: line.description.clone(),
                reference: line.reference.clone(),
                debit_amount: line.debit.to_string(),
                credit_amount: line.credit.to_string(),
                display_debit: i128_to_decimal_str(line.debit, line.asset_scale),
                display_credit: i128_to_decimal_str(line.credit, line.asset_scale),
                running_balance: running.to_string(),
                display_running_balance: i128_to_decimal_str(running, line.asset_scale),
            });
            running -= net_effect;
        }
    }

    let ending_balance = result_lines
        .last()
        .map(|l| l.running_balance.parse::<i128>().unwrap_or(0))
        .unwrap_or(starting_net);

    let next_cursor = if has_more {
        raw_lines.last().map(|l| l.id.clone())
    } else {
        None
    };

    Ok(GeneralLedgerReport {
        account_id: account_id.to_string(),
        account_number: account.account_number,
        account_name: account.name,
        normal_balance: account.normal_balance,
        starting_balance: starting_net.to_string(),
        display_starting_balance: i128_to_decimal_str(starting_net, asset_scale),
        lines: result_lines,
        ending_balance: ending_balance.to_string(),
        display_ending_balance: i128_to_decimal_str(ending_balance, asset_scale),
        has_more,
        next_cursor,
    })
}

fn compute_gl_starting_balance(
    conn: &Connection,
    account_id: &str,
    normal_balance: &str,
    ascending: bool,
    range_start: Option<&str>,
    range_end: Option<&str>,
    cursor: Option<&GlCursor>,
) -> Result<i128, AppError> {
    let (total_debits, total_credits) = if let Some(cursor) = cursor {
        // Sum everything up to (ascending: <=) or before (descending: <) the cursor
        gl_sum_through_cursor(conn, account_id, cursor, ascending)?
    } else if ascending {
        match range_start {
            Some(start) => gl_sum_before_date(conn, account_id, start)?,
            None => (0, 0),
        }
    } else {
        match range_end {
            Some(end) => gl_sum_through_date(conn, account_id, end)?,
            None => gl_sum_all(conn, account_id)?,
        }
    };

    Ok(compute_net(total_debits, total_credits, normal_balance))
}

fn gl_sum_before_date(
    conn: &Connection,
    account_id: &str,
    date: &str,
) -> Result<(i128, i128), AppError> {
    let result: Option<(Option<Vec<u8>>, Option<Vec<u8>>)> = conn
        .query_row(
            "SELECT sum_i128(jel.debit_amount), sum_i128(jel.credit_amount)
             FROM journal_entry_lines jel
             JOIN journal_entries je ON jel.journal_entry_id = je.id
             WHERE jel.account_id = ?1 AND je.entry_date < ?2",
            params![account_id, date],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    match result {
        Some((td, tc)) => Ok((decode_opt(td), decode_opt(tc))),
        None => Ok((0, 0)),
    }
}

fn gl_sum_through_date(
    conn: &Connection,
    account_id: &str,
    date: &str,
) -> Result<(i128, i128), AppError> {
    let result: Option<(Option<Vec<u8>>, Option<Vec<u8>>)> = conn
        .query_row(
            "SELECT sum_i128(jel.debit_amount), sum_i128(jel.credit_amount)
             FROM journal_entry_lines jel
             JOIN journal_entries je ON jel.journal_entry_id = je.id
             WHERE jel.account_id = ?1 AND je.entry_date <= ?2",
            params![account_id, date],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    match result {
        Some((td, tc)) => Ok((decode_opt(td), decode_opt(tc))),
        None => Ok((0, 0)),
    }
}

fn gl_sum_all(
    conn: &Connection,
    account_id: &str,
) -> Result<(i128, i128), AppError> {
    let result: Option<(Option<Vec<u8>>, Option<Vec<u8>>)> = conn
        .query_row(
            "SELECT sum_i128(jel.debit_amount), sum_i128(jel.credit_amount)
             FROM journal_entry_lines jel
             WHERE jel.account_id = ?1",
            params![account_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    match result {
        Some((td, tc)) => Ok((decode_opt(td), decode_opt(tc))),
        None => Ok((0, 0)),
    }
}

/// Sum all debits/credits for lines up to (ascending: <=) or before (descending: <) the cursor.
fn gl_sum_through_cursor(
    conn: &Connection,
    account_id: &str,
    cursor: &GlCursor,
    include_cursor: bool,
) -> Result<(i128, i128), AppError> {
    // Tuple comparison: (entry_date, posted_at, id) <= or < cursor
    let id_op = if include_cursor { "<=" } else { "<" };
    let sql = format!(
        "SELECT sum_i128(jel.debit_amount), sum_i128(jel.credit_amount)
         FROM journal_entry_lines jel
         JOIN journal_entries je ON jel.journal_entry_id = je.id
         WHERE jel.account_id = ?1
           AND (je.entry_date < ?2
                OR (je.entry_date = ?2 AND je.posted_at < ?3)
                OR (je.entry_date = ?2 AND je.posted_at = ?3 AND jel.id {id_op} ?4))"
    );

    let result: Option<(Option<Vec<u8>>, Option<Vec<u8>>)> = conn
        .query_row(
            &sql,
            params![account_id, cursor.entry_date, cursor.posted_at, cursor.line_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    match result {
        Some((td, tc)) => Ok((decode_opt(td), decode_opt(tc))),
        None => Ok((0, 0)),
    }
}

fn fetch_gl_lines(
    conn: &Connection,
    account_id: &str,
    ascending: bool,
    range_start: Option<&str>,
    range_end: Option<&str>,
    cursor: Option<&GlCursor>,
    limit: u32,
) -> Result<Vec<RawGlLine>, AppError> {
    let mut conditions = vec!["jel.account_id = ?1".to_string()];
    let mut param_vals: Vec<Box<dyn rusqlite::types::ToSql>> =
        vec![Box::new(account_id.to_string())];

    if let Some(start) = range_start {
        param_vals.push(Box::new(start.to_string()));
        conditions.push(format!("je.entry_date >= ?{}", param_vals.len()));
    }
    if let Some(end) = range_end {
        param_vals.push(Box::new(end.to_string()));
        conditions.push(format!("je.entry_date <= ?{}", param_vals.len()));
    }

    if let Some(cursor) = cursor {
        param_vals.push(Box::new(cursor.entry_date.clone()));
        let d_idx = param_vals.len();
        param_vals.push(Box::new(cursor.posted_at.clone()));
        let p_idx = param_vals.len();
        param_vals.push(Box::new(cursor.line_id.clone()));
        let i_idx = param_vals.len();

        if ascending {
            conditions.push(format!(
                "(je.entry_date > ?{d} OR (je.entry_date = ?{d} AND je.posted_at > ?{p}) OR (je.entry_date = ?{d} AND je.posted_at = ?{p} AND jel.id > ?{i}))",
                d = d_idx, p = p_idx, i = i_idx
            ));
        } else {
            conditions.push(format!(
                "(je.entry_date < ?{d} OR (je.entry_date = ?{d} AND je.posted_at < ?{p}) OR (je.entry_date = ?{d} AND je.posted_at = ?{p} AND jel.id < ?{i}))",
                d = d_idx, p = p_idx, i = i_idx
            ));
        }
    }

    let order = if ascending {
        "je.entry_date ASC, je.posted_at ASC, jel.id ASC"
    } else {
        "je.entry_date DESC, je.posted_at DESC, jel.id DESC"
    };

    let fetch_limit = limit + 1;
    param_vals.push(Box::new(fetch_limit));
    let limit_idx = param_vals.len();

    let sql = format!(
        "SELECT jel.id, jel.journal_entry_id, je.entry_date, je.description, je.reference,
                jel.debit_amount, jel.credit_amount, c.asset_scale
         FROM journal_entry_lines jel
         JOIN journal_entries je ON jel.journal_entry_id = je.id
         JOIN accounts a ON jel.account_id = a.id
         JOIN currencies c ON a.currency_id = c.id
         WHERE {}
         ORDER BY {}
         LIMIT ?{}",
        conditions.join(" AND "),
        order,
        limit_idx
    );

    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_vals.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let lines = stmt
        .query_map(params_refs.as_slice(), |row| {
            let debit_blob: Vec<u8> = row.get("debit_amount")?;
            let credit_blob: Vec<u8> = row.get("credit_amount")?;
            Ok(RawGlLine {
                id: row.get("id")?,
                journal_entry_id: row.get("journal_entry_id")?,
                entry_date: row.get("entry_date")?,
                description: row.get("description")?,
                reference: row.get("reference")?,
                debit: decode_i128(&debit_blob),
                credit: decode_i128(&credit_blob),
                asset_scale: row.get("asset_scale")?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(lines)
}
