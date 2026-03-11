use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::currency::{CreateCurrencyRequest, Currency, UpdateCurrencyRequest};

pub fn row_to_currency(row: &rusqlite::Row) -> rusqlite::Result<Currency> {
    Ok(Currency {
        id: row.get("id")?,
        code: row.get("code")?,
        name: row.get("name")?,
        symbol: row.get("symbol")?,
        asset_scale: row.get("asset_scale")?,
        asset_type: row.get("asset_type")?,
        caip19_id: row.get("caip19_id")?,
        created_at: row.get("created_at")?,
    })
}

pub fn create_currency(
    conn: &Connection,
    req: CreateCurrencyRequest,
) -> Result<Currency, AppError> {
    if req.code.is_empty() {
        return Err(AppError::ValidationError {
            field: "code".into(),
            message: "Currency code is required".into(),
            suggestion: "Provide a currency code like 'USD' or 'ETH'".into(),
        });
    }
    if req.asset_scale > 18 {
        return Err(AppError::ValidationError {
            field: "asset_scale".into(),
            message: "Asset scale must be between 0 and 18".into(),
            suggestion: "Use a scale between 0 (no decimals) and 18 (wei precision)".into(),
        });
    }
    if !["fiat", "crypto"].contains(&req.asset_type.as_str()) {
        return Err(AppError::ValidationError {
            field: "asset_type".into(),
            message: "Asset type must be 'fiat' or 'crypto'".into(),
            suggestion: "Use 'fiat' for traditional currencies or 'crypto' for digital assets"
                .into(),
        });
    }

    let id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO currencies (id, code, name, symbol, asset_scale, asset_type, caip19_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            req.code,
            req.name,
            req.symbol,
            req.asset_scale,
            req.asset_type,
            req.caip19_id
        ],
    )
    .map_err(|e| match &e {
        rusqlite::Error::SqliteFailure(_, Some(msg)) => {
            if msg.contains("currencies.code") {
                AppError::ValidationError {
                    field: "code".into(),
                    message: format!("Currency code '{}' already exists", req.code),
                    suggestion: "Use a different currency code".into(),
                }
            } else if msg.contains("currencies.caip19_id") {
                AppError::ValidationError {
                    field: "caip19_id".into(),
                    message: format!("CAIP-19 ID '{}' already exists", req.caip19_id),
                    suggestion: "Use a different CAIP-19 identifier".into(),
                }
            } else {
                AppError::from(e)
            }
        }
        _ => AppError::from(e),
    })?;

    get_currency(conn, &id)
}

pub fn get_currency(conn: &Connection, id: &str) -> Result<Currency, AppError> {
    conn.query_row(
        "SELECT id, code, name, symbol, asset_scale, asset_type, caip19_id, created_at
         FROM currencies WHERE id = ?1",
        params![id],
        row_to_currency,
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound {
        resource: "Currency".into(),
        id: id.into(),
    })
}

pub fn list_currencies(
    conn: &Connection,
    limit: u32,
    cursor: Option<&str>,
) -> Result<(Vec<Currency>, bool, Option<String>), AppError> {
    let fetch_limit = limit + 1;

    let mut currencies = if let Some(cursor) = cursor {
        let mut stmt = conn.prepare(
            "SELECT id, code, name, symbol, asset_scale, asset_type, caip19_id, created_at
             FROM currencies WHERE id > ?1 ORDER BY id LIMIT ?2",
        )?;
        stmt.query_map(params![cursor, fetch_limit], row_to_currency)?
            .collect::<Result<Vec<_>, _>>()?
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, code, name, symbol, asset_scale, asset_type, caip19_id, created_at
             FROM currencies ORDER BY id LIMIT ?1",
        )?;
        stmt.query_map(params![fetch_limit], row_to_currency)?
            .collect::<Result<Vec<_>, _>>()?
    };

    let has_more = currencies.len() > limit as usize;
    if has_more {
        currencies.truncate(limit as usize);
    }
    let next_cursor = if has_more {
        currencies.last().map(|c| c.id.clone())
    } else {
        None
    };

    Ok((currencies, has_more, next_cursor))
}

pub fn update_currency(
    conn: &Connection,
    id: &str,
    req: UpdateCurrencyRequest,
) -> Result<Currency, AppError> {
    let _ = get_currency(conn, id)?;

    if let Some(ref name) = req.name {
        conn.execute(
            "UPDATE currencies SET name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
    }
    if let Some(ref symbol) = req.symbol {
        conn.execute(
            "UPDATE currencies SET symbol = ?1 WHERE id = ?2",
            params![symbol, id],
        )?;
    }

    get_currency(conn, id)
}
