use rusqlite::Connection;

use crate::error::AppError;
use crate::models::account::{AccountFilters, CreateAccountRequest};
use crate::services::account_service;

pub fn list(
    conn: &Connection,
    account_type: Option<&str>,
    currency_id: Option<&str>,
    json: bool,
) -> Result<(), AppError> {
    let filters = AccountFilters {
        account_type: account_type.map(String::from),
        currency_id: currency_id.map(String::from),
        is_active: Some(true),
        parent_id: None,
        limit: Some(200),
        cursor: None,
    };
    let (accounts, _, _) = account_service::list_accounts(conn, &filters)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&accounts)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
        return Ok(());
    }

    if accounts.is_empty() {
        println!("No accounts found.");
        return Ok(());
    }

    println!(
        "{:<36}  {:<8}  {:<28}  {:<10}  {:<8}  {}",
        "ID", "NUMBER", "NAME", "TYPE", "NORMAL", "SUBLEDGER"
    );
    println!("{}", "-".repeat(110));
    for a in &accounts {
        println!(
            "{:<36}  {:<8}  {:<28}  {:<10}  {:<8}  {}",
            a.id,
            a.account_number,
            a.name,
            a.account_type,
            a.normal_balance,
            if a.has_subledger { "yes" } else { "" }
        );
    }
    println!("\n{} account(s)", accounts.len());
    Ok(())
}

pub fn create(
    conn: &Connection,
    name: &str,
    currency_id: &str,
    account_type: &str,
    normal_balance: &str,
    number: &str,
    has_subledger: bool,
    parent_id: Option<&str>,
    entity_id: Option<&str>,
    json: bool,
) -> Result<(), AppError> {
    let req = CreateAccountRequest {
        currency_id: Some(currency_id.to_string()),
        account_number: number.to_string(),
        name: name.to_string(),
        account_type: Some(account_type.to_string()),
        normal_balance: Some(normal_balance.to_string()),
        has_subledger,
        parent_id: parent_id.map(String::from),
        entity_id: entity_id.map(String::from),
        xbrl_tag: None,
    };
    let account = account_service::create_account(conn, req)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&account)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
    } else {
        println!(
            "Created account: {} - {} [{}]",
            account.account_number, account.name, account.id
        );
    }
    Ok(())
}

pub fn get(conn: &Connection, id: &str, json: bool) -> Result<(), AppError> {
    let account = account_service::get_account(conn, id)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&account)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
    } else {
        println!("ID:             {}", account.id);
        println!("Number:         {}", account.account_number);
        println!("Name:           {}", account.name);
        println!("Type:           {}", account.account_type);
        println!("Normal Balance: {}", account.normal_balance);
        println!("Currency:       {}", account.currency_id);
        println!("Subledger:      {}", account.has_subledger);
        println!("Active:         {}", account.is_active);
        if let Some(ref pid) = account.parent_id {
            println!("Parent:         {}", pid);
        }
        if let Some(ref eid) = account.entity_id {
            println!("Entity:         {}", eid);
        }
        println!("Created:        {}", account.created_at);
    }
    Ok(())
}
