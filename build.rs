use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use iso_currency::{Currency, IntoEnumIterator};

fn main() {
    // --- Step 1: Generate fiat-currencies.json ---
    generate_fiat_json();

    // --- Step 2: Fetch crypto-tokens.json from Uniswap token list ---
    fetch_crypto_tokens_json();

    // --- Step 3: Build the frontend ---
    build_frontend();
}

fn generate_fiat_json() {
    let out_path = Path::new("frontend/src/lib/data/fiat-currencies.json");

    // Only regenerate if the output doesn't exist (the iso_currency crate data is static)
    if out_path.exists() {
        return;
    }

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create fiat-currencies.json output directory");
    }

    #[derive(serde::Serialize)]
    struct FiatCurrency {
        code: String,
        name: String,
        symbol: String,
        asset_scale: u16,
        caip19_id: String,
    }

    let currencies: Vec<FiatCurrency> = Currency::iter()
        .filter(|c| {
            let code = c.code();
            let is_special_x =
                code.starts_with('X') && !matches!(code, "XAF" | "XCD" | "XOF" | "XPF");
            let has_exponent = c.exponent().is_some();
            let has_users = !c.used_by().is_empty();
            !is_special_x && has_exponent && has_users
        })
        .map(|c| {
            let code = c.code().to_string();
            FiatCurrency {
                caip19_id: format!("swift:0/iso4217:{code}"),
                code,
                name: c.name().to_string(),
                symbol: c.symbol().symbol.clone(),
                asset_scale: c.exponent().unwrap_or(2),
            }
        })
        .collect();

    let json =
        serde_json::to_string_pretty(&currencies).expect("Failed to serialize fiat currencies");
    fs::write(out_path, json).expect("Failed to write fiat-currencies.json");

    println!("cargo:warning=Generated fiat-currencies.json ({} currencies)", currencies.len());
}

fn fetch_crypto_tokens_json() {
    let out_path = Path::new("frontend/src/lib/data/crypto-tokens.json");

    // Only fetch if the output doesn't exist (delete the file to refresh)
    if out_path.exists() {
        return;
    }

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create crypto-tokens.json output directory");
    }

    println!("cargo:warning=Fetching Uniswap Default token list...");

    let body: String = match ureq::get("https://tokens.uniswap.org").call() {
        Ok(mut resp) => resp.body_mut().read_to_string().expect("Failed to read response body"),
        Err(e) => {
            println!("cargo:warning=Failed to fetch token list ({e}), skipping crypto-tokens.json generation");
            return;
        }
    };

    let token_list: serde_json::Value =
        serde_json::from_str(&body).expect("Failed to parse token list JSON");

    #[derive(serde::Serialize)]
    struct CryptoToken {
        code: String,
        name: String,
        symbol: String,
        asset_scale: u64,
        caip19_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        logo_uri: Option<String>,
    }

    let mut tokens: Vec<CryptoToken> = token_list["tokens"]
        .as_array()
        .expect("tokens field is not an array")
        .iter()
        .filter(|t| t["chainId"].as_u64() == Some(1)) // Ethereum mainnet only
        .map(|t| {
            let address = t["address"].as_str().unwrap_or_default();
            let symbol = t["symbol"].as_str().unwrap_or_default().to_string();
            CryptoToken {
                code: symbol.clone(),
                name: t["name"].as_str().unwrap_or_default().to_string(),
                symbol,
                asset_scale: t["decimals"].as_u64().unwrap_or(18),
                caip19_id: format!("eip155:1/erc20:{address}"),
                logo_uri: t["logoURI"].as_str().map(|s| s.to_string()),
            }
        })
        .collect();

    tokens.sort_by(|a, b| a.code.cmp(&b.code));

    // Deduplicate by symbol (keep first occurrence)
    let mut seen = std::collections::HashSet::new();
    tokens.retain(|t| seen.insert(t.code.clone()));

    let json =
        serde_json::to_string_pretty(&tokens).expect("Failed to serialize crypto tokens");
    fs::write(out_path, json).expect("Failed to write crypto-tokens.json");

    println!(
        "cargo:warning=Generated crypto-tokens.json ({} tokens)",
        tokens.len()
    );
}

fn pnpm_cmd() -> Command {
    if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "pnpm"]);
        cmd
    } else {
        Command::new("pnpm")
    }
}

fn build_frontend() {
    let frontend_dir = Path::new("frontend");

    // Skip if no frontend directory
    if !frontend_dir.join("package.json").exists() {
        println!("cargo:warning=No frontend/package.json found, skipping frontend build");
        return;
    }

    // Rerun if frontend source changes
    println!("cargo:rerun-if-changed=frontend/src/");
    println!("cargo:rerun-if-changed=frontend/static/");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/svelte.config.js");
    println!("cargo:rerun-if-changed=frontend/vite.config.ts");

    // Skip frontend build in test mode or if SKIP_FRONTEND_BUILD is set
    if env::var("SKIP_FRONTEND_BUILD").is_ok() || env::var("CARGO_CFG_TEST").is_ok() {
        return;
    }

    // Install dependencies if node_modules is missing
    if !frontend_dir.join("node_modules").exists() {
        println!("cargo:warning=Installing frontend dependencies...");
        let status = pnpm_cmd()
            .arg("install")
            .current_dir(frontend_dir)
            .status();

        match status {
            Ok(s) if s.success() => {}
            Ok(s) => {
                println!("cargo:warning=pnpm install failed with status {s}, skipping frontend build");
                return;
            }
            Err(e) => {
                println!("cargo:warning=pnpm not found ({e}), skipping frontend build");
                return;
            }
        }
    }

    // Build frontend
    println!("cargo:warning=Building frontend...");
    let status = pnpm_cmd()
        .args(["run", "build"])
        .current_dir(frontend_dir)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:warning=Frontend build complete");
        }
        Ok(s) => {
            panic!("Frontend build failed with status {s}");
        }
        Err(e) => {
            println!("cargo:warning=Failed to run pnpm build ({e}), skipping frontend build");
        }
    }
}
