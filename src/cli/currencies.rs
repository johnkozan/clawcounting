use rusqlite::Connection;

use crate::error::AppError;
use crate::models::currency::CreateCurrencyRequest;
use crate::services::currency_service;

pub fn list(conn: &Connection, json: bool) -> Result<(), AppError> {
    let (currencies, _, _) = currency_service::list_currencies(conn, 200, None)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&currencies)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
        return Ok(());
    }

    if currencies.is_empty() {
        println!("No currencies found.");
        return Ok(());
    }

    println!(
        "{:<36}  {:<8}  {:<24}  {:<6}  {:<6}  {}",
        "ID", "CODE", "NAME", "SCALE", "TYPE", "CAIP-19"
    );
    println!("{}", "-".repeat(110));
    for c in &currencies {
        println!(
            "{:<36}  {:<8}  {:<24}  {:<6}  {:<6}  {}",
            c.id, c.code, c.name, c.asset_scale, c.asset_type, c.caip19_id
        );
    }
    println!("\n{} currency(ies)", currencies.len());
    Ok(())
}

pub fn create(
    conn: &Connection,
    code: &str,
    name: &str,
    symbol: &str,
    asset_scale: u32,
    asset_type: &str,
    caip19: &str,
    json: bool,
) -> Result<(), AppError> {
    let req = CreateCurrencyRequest {
        code: code.to_string(),
        name: name.to_string(),
        symbol: symbol.to_string(),
        asset_scale,
        asset_type: asset_type.to_string(),
        caip19_id: caip19.to_string(),
    };
    let currency = currency_service::create_currency(conn, req)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&currency)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
    } else {
        println!("Created currency: {} ({}) [{}]", currency.code, currency.name, currency.id);
    }
    Ok(())
}

pub fn create_fiat(conn: &Connection, code: &str, json: bool) -> Result<(), AppError> {
    let iso = iso_currency::Currency::from_code(code).ok_or_else(|| AppError::ValidationError {
        field: "code".into(),
        message: format!("Unknown ISO 4217 currency code: {code}"),
        suggestion: "Use a valid ISO 4217 code like USD, EUR, GBP, JPY".into(),
    })?;

    let name = iso.name().to_string();
    let symbol = fiat_symbol(code).to_string();
    let asset_scale = iso.exponent().unwrap_or(2) as u32;
    let caip19 = format!("swift:0/iso4217:{code}");

    create(
        conn, code, &name, &symbol, asset_scale, "fiat", &caip19, json,
    )
}

pub fn get(conn: &Connection, id: &str, json: bool) -> Result<(), AppError> {
    let currency = currency_service::get_currency(conn, id)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&currency)
                .map_err(|e| AppError::Internal(e.to_string()))?
        );
    } else {
        println!("ID:          {}", currency.id);
        println!("Code:        {}", currency.code);
        println!("Name:        {}", currency.name);
        println!("Symbol:      {}", currency.symbol);
        println!("Asset Scale: {}", currency.asset_scale);
        println!("Type:        {}", currency.asset_type);
        println!("CAIP-19:     {}", currency.caip19_id);
        println!("Created:     {}", currency.created_at);
    }
    Ok(())
}

fn fiat_symbol(code: &str) -> &str {
    match code {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" => "¥",
        "CNY" => "¥",
        "KRW" => "₩",
        "INR" => "₹",
        "RUB" => "₽",
        "BRL" => "R$",
        "CHF" => "CHF",
        "CAD" => "CA$",
        "AUD" => "A$",
        "MXN" => "MX$",
        "SGD" => "S$",
        "HKD" => "HK$",
        "SEK" => "kr",
        "NOK" => "kr",
        "DKK" => "kr",
        "PLN" => "zł",
        "THB" => "฿",
        "ZAR" => "R",
        "TRY" => "₺",
        "ILS" => "₪",
        _ => code,
    }
}
