use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::account::{Account, AccountFilters, CreateAccountRequest, UpdateAccountRequest};
use crate::services::currency_service;

const VALID_TYPES: &[&str] = &["asset", "liability", "equity", "revenue", "expense"];
const VALID_BALANCES: &[&str] = &["debit", "credit"];

fn row_to_account(row: &rusqlite::Row) -> rusqlite::Result<Account> {
    Ok(Account {
        id: row.get("id")?,
        currency_id: row.get("currency_id")?,
        account_number: row.get("account_number")?,
        name: row.get("name")?,
        account_type: row.get("account_type")?,
        normal_balance: row.get("normal_balance")?,
        has_subledger: row.get::<_, i32>("has_subledger")? != 0,
        parent_id: row.get("parent_id")?,
        entity_id: row.get("entity_id")?,
        xbrl_tag: row.get("xbrl_tag")?,
        is_active: row.get::<_, i32>("is_active")? != 0,
        created_at: row.get("created_at")?,
    })
}

pub fn create_account(
    conn: &Connection,
    req: CreateAccountRequest,
) -> Result<Account, AppError> {
    let (currency_id, account_type, normal_balance);

    if let Some(ref parent_id) = req.parent_id {
        // Sub-account: inherit from parent
        let parent = get_account(conn, parent_id)?;
        if !parent.has_subledger {
            return Err(AppError::ValidationError {
                field: "parent_id".into(),
                message: "Parent account does not have subledger enabled".into(),
                suggestion: "Set has_subledger=true on the parent account first".into(),
            });
        }
        if req.entity_id.is_none() {
            return Err(AppError::ValidationError {
                field: "entity_id".into(),
                message: "entity_id is required for sub-accounts".into(),
                suggestion: "Provide an entity_id to identify this sub-account".into(),
            });
        }
        if req.has_subledger {
            return Err(AppError::ValidationError {
                field: "has_subledger".into(),
                message: "Sub-accounts cannot have subledgers".into(),
                suggestion: "Remove has_subledger or remove parent_id".into(),
            });
        }
        currency_id = parent.currency_id.clone();
        account_type = parent.account_type.clone();
        normal_balance = parent.normal_balance.clone();
    } else {
        // Top-level account
        if req.entity_id.is_some() {
            return Err(AppError::ValidationError {
                field: "entity_id".into(),
                message: "entity_id requires parent_id".into(),
                suggestion: "Provide parent_id along with entity_id, or remove entity_id".into(),
            });
        }

        currency_id = req.currency_id.clone().ok_or_else(|| AppError::ValidationError {
            field: "currency_id".into(),
            message: "currency_id is required for top-level accounts".into(),
            suggestion: "Provide the ID of the currency for this account".into(),
        })?;

        // Validate currency exists
        currency_service::get_currency(conn, &currency_id)?;

        account_type = req.account_type.clone().ok_or_else(|| AppError::ValidationError {
            field: "account_type".into(),
            message: "account_type is required for top-level accounts".into(),
            suggestion: "Use one of: asset, liability, equity, revenue, expense".into(),
        })?;
        if !VALID_TYPES.contains(&account_type.as_str()) {
            return Err(AppError::ValidationError {
                field: "account_type".into(),
                message: format!("Invalid account type: {account_type}"),
                suggestion: "Use one of: asset, liability, equity, revenue, expense".into(),
            });
        }

        normal_balance = req.normal_balance.clone().ok_or_else(|| AppError::ValidationError {
            field: "normal_balance".into(),
            message: "normal_balance is required for top-level accounts".into(),
            suggestion: "Use 'debit' or 'credit'".into(),
        })?;
        if !VALID_BALANCES.contains(&normal_balance.as_str()) {
            return Err(AppError::ValidationError {
                field: "normal_balance".into(),
                message: format!("Invalid normal balance: {normal_balance}"),
                suggestion: "Use 'debit' or 'credit'".into(),
            });
        }
    }

    let id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO accounts (id, currency_id, account_number, name, account_type, normal_balance,
         has_subledger, parent_id, entity_id, xbrl_tag)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            currency_id,
            req.account_number,
            req.name,
            account_type,
            normal_balance,
            req.has_subledger as i32,
            req.parent_id,
            req.entity_id,
            req.xbrl_tag,
        ],
    )
    .map_err(|e| match &e {
        rusqlite::Error::SqliteFailure(_, Some(msg)) => {
            if msg.contains("accounts.account_number") {
                AppError::ValidationError {
                    field: "account_number".into(),
                    message: format!(
                        "Account number '{}' already exists",
                        req.account_number
                    ),
                    suggestion: "Use a different account number".into(),
                }
            } else {
                AppError::from(e)
            }
        }
        _ => AppError::from(e),
    })?;

    get_account(conn, &id)
}

pub fn get_account(conn: &Connection, id: &str) -> Result<Account, AppError> {
    conn.query_row(
        "SELECT id, currency_id, account_number, name, account_type, normal_balance,
                has_subledger, parent_id, entity_id, xbrl_tag, is_active, created_at
         FROM accounts WHERE id = ?1",
        params![id],
        row_to_account,
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound {
        resource: "Account".into(),
        id: id.into(),
    })
}

pub fn list_accounts(
    conn: &Connection,
    filters: &AccountFilters,
) -> Result<(Vec<Account>, bool, Option<String>), AppError> {
    let limit = filters.limit();
    let fetch_limit = limit + 1;

    let mut conditions = vec!["1=1".to_string()];
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref at) = filters.account_type {
        param_values.push(Box::new(at.clone()));
        conditions.push(format!("account_type = ?{}", param_values.len()));
    }
    if let Some(ref cid) = filters.currency_id {
        param_values.push(Box::new(cid.clone()));
        conditions.push(format!("currency_id = ?{}", param_values.len()));
    }
    if let Some(active) = filters.is_active {
        param_values.push(Box::new(active as i32));
        conditions.push(format!("is_active = ?{}", param_values.len()));
    }
    if let Some(ref pid) = filters.parent_id {
        param_values.push(Box::new(pid.clone()));
        conditions.push(format!("parent_id = ?{}", param_values.len()));
    }
    if let Some(ref cursor) = filters.cursor {
        param_values.push(Box::new(cursor.clone()));
        conditions.push(format!("id > ?{}", param_values.len()));
    }

    param_values.push(Box::new(fetch_limit));
    let limit_param = param_values.len();

    let sql = format!(
        "SELECT id, currency_id, account_number, name, account_type, normal_balance,
                has_subledger, parent_id, entity_id, xbrl_tag, is_active, created_at
         FROM accounts WHERE {} ORDER BY id LIMIT ?{}",
        conditions.join(" AND "),
        limit_param
    );

    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let mut accounts = stmt
        .query_map(params_refs.as_slice(), row_to_account)?
        .collect::<Result<Vec<_>, _>>()?;

    let has_more = accounts.len() > limit as usize;
    if has_more {
        accounts.truncate(limit as usize);
    }
    let next_cursor = if has_more {
        accounts.last().map(|a| a.id.clone())
    } else {
        None
    };

    Ok((accounts, has_more, next_cursor))
}

pub fn update_account(
    conn: &Connection,
    id: &str,
    req: UpdateAccountRequest,
) -> Result<Account, AppError> {
    let _ = get_account(conn, id)?;

    if let Some(ref name) = req.name {
        conn.execute(
            "UPDATE accounts SET name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
    }
    if let Some(active) = req.is_active {
        conn.execute(
            "UPDATE accounts SET is_active = ?1 WHERE id = ?2",
            params![active as i32, id],
        )?;
    }
    if let Some(ref xbrl) = req.xbrl_tag {
        conn.execute(
            "UPDATE accounts SET xbrl_tag = ?1 WHERE id = ?2",
            params![xbrl, id],
        )?;
    }

    get_account(conn, id)
}

pub fn get_sub_accounts(conn: &Connection, parent_id: &str) -> Result<Vec<Account>, AppError> {
    // Verify parent exists
    let _ = get_account(conn, parent_id)?;

    let mut stmt = conn.prepare(
        "SELECT id, currency_id, account_number, name, account_type, normal_balance,
                has_subledger, parent_id, entity_id, xbrl_tag, is_active, created_at
         FROM accounts WHERE parent_id = ?1 ORDER BY account_number",
    )?;
    let accounts = stmt
        .query_map(params![parent_id], row_to_account)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(accounts)
}
