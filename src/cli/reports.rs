use rusqlite::Connection;

use crate::error::AppError;
use crate::models::report::*;
use crate::services::report_service;

pub fn trial_balance(
    conn: &Connection,
    period_id: Option<&str>,
    currency_id: Option<&str>,
    json: bool,
) -> Result<(), AppError> {
    let query = TrialBalanceQuery {
        period_id: period_id.map(String::from),
        currency_id: currency_id.map(String::from),
    };
    let report = report_service::trial_balance(conn, &query)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
        return Ok(());
    }

    println!("=== Trial Balance ===");
    if let Some(ref pid) = report.period_id {
        println!("Period: {pid}");
    } else {
        println!("Period: All time");
    }
    println!();
    println!(
        "{:<10}  {:<30}  {:<12}  {:>16}  {:>16}",
        "NUMBER", "ACCOUNT", "TYPE", "DEBITS", "CREDITS"
    );
    println!("{}", "-".repeat(90));

    for row in &report.rows {
        println!(
            "{:<10}  {:<30}  {:<12}  {:>16}  {:>16}",
            row.account_number,
            row.account_name,
            row.account_type,
            row.display_debit_total,
            row.display_credit_total,
        );
    }

    println!("{}", "-".repeat(90));
    println!(
        "{:<10}  {:<30}  {:<12}  {:>16}  {:>16}",
        "", "TOTALS", "", report.display_grand_total_debits, report.display_grand_total_credits,
    );
    let status = if report.is_balanced {
        "BALANCED"
    } else {
        "*** UNBALANCED ***"
    };
    println!("\nStatus: {status}");
    Ok(())
}

pub fn balance_sheet(
    conn: &Connection,
    period_id: Option<&str>,
    as_of_date: Option<&str>,
    json: bool,
) -> Result<(), AppError> {
    let query = BalanceSheetQuery {
        period_id: period_id.map(String::from),
        as_of_date: as_of_date.map(String::from),
    };
    let report = report_service::balance_sheet(conn, &query)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
        return Ok(());
    }

    println!("=== Balance Sheet ===");
    if let Some(ref d) = report.as_of_date {
        println!("As of: {d}");
    } else if let Some(ref pid) = report.period_id {
        println!("Through period: {pid}");
    } else {
        println!("As of: Current");
    }
    println!();

    print_bs_section(&report.assets);
    print_bs_section(&report.liabilities);
    print_bs_section(&report.equity);

    println!("{}", "-".repeat(60));
    println!("{:<40}  {:>16}", "Total Assets", report.display_total_assets);
    println!(
        "{:<40}  {:>16}",
        "Total Liabilities + Equity", report.display_total_liabilities_and_equity
    );
    let status = if report.is_balanced {
        "BALANCED"
    } else {
        "*** UNBALANCED ***"
    };
    println!("\nStatus: {status}");
    Ok(())
}

fn print_bs_section(section: &BalanceSheetSection) {
    println!("--- {} ---", section.label);
    for row in &section.accounts {
        println!(
            "  {:<8}  {:<28}  {:>16}",
            row.account_number, row.account_name, row.display_balance,
        );
    }
    println!(
        "  {:<8}  {:<28}  {:>16}",
        "", format!("Total {}", section.label), section.display_total,
    );
    println!();
}

pub fn income_statement(
    conn: &Connection,
    period_id: &str,
    json: bool,
) -> Result<(), AppError> {
    let query = IncomeStatementQuery {
        period_id: Some(period_id.to_string()),
    };
    let report = report_service::income_statement(conn, &query)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
        return Ok(());
    }

    println!("=== Income Statement ===");
    println!("Period: {}", report.period_id);
    println!();

    println!("--- Revenue ---");
    for row in &report.revenue {
        println!(
            "  {:<8}  {:<28}  {:>16}",
            row.account_number, row.account_name, row.display_amount,
        );
    }
    println!(
        "  {:<8}  {:<28}  {:>16}",
        "", "Total Revenue", report.display_total_revenue,
    );
    println!();

    println!("--- Expenses ---");
    for row in &report.expenses {
        println!(
            "  {:<8}  {:<28}  {:>16}",
            row.account_number, row.account_name, row.display_amount,
        );
    }
    println!(
        "  {:<8}  {:<28}  {:>16}",
        "", "Total Expenses", report.display_total_expenses,
    );
    println!();

    println!("{}", "-".repeat(60));
    println!(
        "{:<40}  {:>16}",
        "Net Income", report.display_net_income,
    );
    Ok(())
}

pub fn general_ledger(
    conn: &Connection,
    account_id: &str,
    period_id: Option<&str>,
    start_date: Option<&str>,
    end_date: Option<&str>,
    json: bool,
) -> Result<(), AppError> {
    let query = GeneralLedgerQuery {
        account_id: Some(account_id.to_string()),
        period_id: period_id.map(String::from),
        start_date: start_date.map(String::from),
        end_date: end_date.map(String::from),
        sort: None,
        limit: None,
        cursor: None,
    };
    let report = report_service::general_ledger(conn, &query)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
        return Ok(());
    }

    println!(
        "=== General Ledger: {} ({}) ===",
        report.account_name, report.account_number
    );
    println!("Starting Balance: {}", report.display_starting_balance);
    println!();
    println!(
        "{:<12}  {:<30}  {:>14}  {:>14}  {:>16}",
        "DATE", "DESCRIPTION", "DEBIT", "CREDIT", "BALANCE"
    );
    println!("{}", "-".repeat(92));

    for line in &report.lines {
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
        let desc = if line.description.len() > 28 {
            format!("{}...", &line.description[..25])
        } else {
            line.description.clone()
        };
        println!(
            "{:<12}  {:<30}  {:>14}  {:>14}  {:>16}",
            line.entry_date, desc, dr, cr, line.display_running_balance,
        );
    }

    println!("{}", "-".repeat(92));
    println!("Ending Balance: {}", report.display_ending_balance);
    if report.has_more {
        println!("(more entries available — use --json for cursor-based pagination)");
    }
    Ok(())
}
