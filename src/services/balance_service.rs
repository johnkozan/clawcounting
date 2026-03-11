use rusqlite::{Connection, OptionalExtension, params};

use crate::db::i128_funcs::decode_i128;
use crate::error::AppError;
use crate::models::amount::i128_to_decimal_str;
use crate::models::journal_entry::BalanceResponse;
use crate::services::account_service;

pub fn get_account_balance(
    conn: &Connection,
    account_id: &str,
    period_id: Option<&str>,
) -> Result<BalanceResponse, AppError> {
    let account = account_service::get_account(conn, account_id)?;

    let asset_scale: u32 = conn.query_row(
        "SELECT asset_scale FROM currencies WHERE id = ?1",
        params![account.currency_id],
        |row| row.get(0),
    )?;

    let (total_debits, total_credits) = if account.has_subledger {
        // Control account: sum all children's balances
        get_control_account_balance(conn, account_id, period_id)?
    } else if let Some(period_id) = period_id {
        // Specific period
        get_period_balance(conn, account_id, period_id)?
    } else {
        // All periods
        get_all_periods_balance(conn, account_id)?
    };

    let net_balance = if account.normal_balance == "debit" {
        total_debits - total_credits
    } else {
        total_credits - total_debits
    };

    Ok(BalanceResponse {
        account_id: account_id.to_string(),
        period_id: period_id.map(String::from),
        total_debits: total_debits.to_string(),
        total_credits: total_credits.to_string(),
        net_balance: net_balance.to_string(),
        display_debits: i128_to_decimal_str(total_debits, asset_scale),
        display_credits: i128_to_decimal_str(total_credits, asset_scale),
        display_balance: i128_to_decimal_str(net_balance, asset_scale),
    })
}

fn get_period_balance(
    conn: &Connection,
    account_id: &str,
    period_id: &str,
) -> Result<(i128, i128), AppError> {
    let result: Option<(Vec<u8>, Vec<u8>)> = conn
        .query_row(
            "SELECT total_debits, total_credits FROM account_balances
             WHERE account_id = ?1 AND period_id = ?2",
            params![account_id, period_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    match result {
        Some((td, tc)) => Ok((decode_i128(&td), decode_i128(&tc))),
        None => Ok((0, 0)),
    }
}

fn get_all_periods_balance(
    conn: &Connection,
    account_id: &str,
) -> Result<(i128, i128), AppError> {
    let result: Option<(Option<Vec<u8>>, Option<Vec<u8>>)> = conn
        .query_row(
            "SELECT sum_i128(total_debits), sum_i128(total_credits)
             FROM account_balances WHERE account_id = ?1",
            params![account_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    match result {
        Some((Some(td), Some(tc))) => Ok((decode_i128(&td), decode_i128(&tc))),
        _ => Ok((0, 0)),
    }
}

fn get_control_account_balance(
    conn: &Connection,
    parent_id: &str,
    period_id: Option<&str>,
) -> Result<(i128, i128), AppError> {
    let (td, tc) = if let Some(period_id) = period_id {
        let result: Option<(Option<Vec<u8>>, Option<Vec<u8>>)> = conn
            .query_row(
                "SELECT sum_i128(ab.total_debits), sum_i128(ab.total_credits)
                 FROM account_balances ab
                 JOIN accounts a ON ab.account_id = a.id
                 WHERE a.parent_id = ?1 AND ab.period_id = ?2",
                params![parent_id, period_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        result.unwrap_or((None, None))
    } else {
        let result: Option<(Option<Vec<u8>>, Option<Vec<u8>>)> = conn
            .query_row(
                "SELECT sum_i128(ab.total_debits), sum_i128(ab.total_credits)
                 FROM account_balances ab
                 JOIN accounts a ON ab.account_id = a.id
                 WHERE a.parent_id = ?1",
                params![parent_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        result.unwrap_or((None, None))
    };

    Ok((
        td.map(|b| decode_i128(&b)).unwrap_or(0),
        tc.map(|b| decode_i128(&b)).unwrap_or(0),
    ))
}
