use axum::http::{HeaderName, HeaderValue};
use axum_test::TestServer;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};

use clawcounting::app_state::AppState;
use clawcounting::config::Config;
use clawcounting::db::connection::setup_connection;
use clawcounting::db::migrations::run_migrations;
use clawcounting::db::pool::DbPools;
use clawcounting::router::build_router;
use clawcounting::services::user_service;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Full test application with HTTP server and temp database.
pub struct TestApp {
    pub server: TestServer,
    pub db_path: String,
    pub auth_token: String,
    pub test_user_id: String,
}

impl Drop for TestApp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.db_path);
        // Also remove WAL/SHM files
        let _ = std::fs::remove_file(format!("{}-wal", self.db_path));
        let _ = std::fs::remove_file(format!("{}-shm", self.db_path));
    }
}

impl TestApp {
    /// Create a new test app with a fresh database.
    pub async fn new() -> Self {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let db_path = format!(
            "{}/clawcounting_test_{n}_{}.db",
            std::env::temp_dir().display(),
            std::process::id()
        );

        // Bootstrap DB
        let mut conn = setup_connection(&db_path).expect("Failed to open test database");
        run_migrations(&mut conn).expect("Failed to run test migrations");

        // Create a test admin user with an API key for authentication
        let api_key = format!("tsk_test_admin_{n}_{}", std::process::id());
        let key_hash = user_service::hash_api_key(&api_key);
        let test_user_id = uuid::Uuid::now_v7().to_string();
        conn.execute(
            "INSERT INTO users (id, name, email, api_key_hash, permissions, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, 1)",
            rusqlite::params![
                &test_user_id,
                "Test Admin",
                format!("admin_{n}_{}@test.local", std::process::id()),
                &key_hash,
                r#"{"admin":true}"#,
            ],
        )
        .expect("Failed to create test admin user");

        drop(conn);

        // Create pools
        let pools = DbPools::new(&db_path, 2).expect("Failed to create test pools");
        let config = Config {
            db_path: db_path.clone(),
            port: 0,
            jwt_secret: Some("test-secret".into()),
        };
        let state = AppState {
            pools,
            config,
        };
        let app = build_router(state);
        let server = TestServer::new(app);

        TestApp {
            server,
            db_path,
            auth_token: api_key,
            test_user_id,
        }
    }

    /// Create a test app with NO users (for testing setup flow).
    pub async fn new_without_user() -> Self {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let db_path = format!(
            "{}/clawcounting_test_{n}_{}.db",
            std::env::temp_dir().display(),
            std::process::id()
        );

        let mut conn = setup_connection(&db_path).expect("Failed to open test database");
        run_migrations(&mut conn).expect("Failed to run test migrations");
        drop(conn);

        let pools = DbPools::new(&db_path, 2).expect("Failed to create test pools");
        let config = Config {
            db_path: db_path.clone(),
            port: 0,
            jwt_secret: Some("test-secret".into()),
        };
        let state = AppState {
            pools,
            config,
        };
        let app = build_router(state);
        let server = TestServer::new(app);

        TestApp {
            server,
            db_path,
            auth_token: String::new(),
            test_user_id: String::new(),
        }
    }

    // ── Auth helpers ─────────────────────────────────────────────────

    pub fn auth_name(&self) -> HeaderName {
        HeaderName::from_static("authorization")
    }

    pub fn auth_value(&self) -> HeaderValue {
        HeaderValue::from_str(&format!("Bearer {}", self.auth_token)).unwrap()
    }

    // ── Helpers ────────────────────────────────────────────────────

    pub async fn create_currency(
        &self,
        code: &str,
        name: &str,
        symbol: &str,
        scale: u32,
        asset_type: &str,
    ) -> Value {
        self.server
            .post("/api/v1/currencies")
            .add_header(self.auth_name(), self.auth_value())
            .json(&json!({
                "code": code,
                "name": name,
                "symbol": symbol,
                "asset_scale": scale,
                "asset_type": asset_type,
                "caip19_id": format!("swift:0/iso4217:{code}")
            }))
            .await
            .json::<Value>()
    }

    pub async fn create_usd(&self) -> String {
        let resp = self
            .create_currency("USD", "US Dollar", "$", 2, "fiat")
            .await;
        resp["data"]["id"].as_str().unwrap().to_string()
    }

    pub async fn create_account(
        &self,
        currency_id: &str,
        number: &str,
        name: &str,
        account_type: &str,
        normal_balance: &str,
    ) -> Value {
        self.server
            .post("/api/v1/accounts")
            .add_header(self.auth_name(), self.auth_value())
            .json(&json!({
                "currency_id": currency_id,
                "account_number": number,
                "name": name,
                "account_type": account_type,
                "normal_balance": normal_balance,
            }))
            .await
            .json::<Value>()
    }

    /// Create a minimal chart of accounts. Returns (cash_id, revenue_id, expense_id, re_id).
    pub async fn create_test_accounts(&self, currency_id: &str) -> (String, String, String, String) {
        let cash = self
            .create_account(currency_id, "1000", "Cash", "asset", "debit")
            .await;
        let revenue = self
            .create_account(currency_id, "4000", "Revenue", "revenue", "credit")
            .await;
        let expense = self
            .create_account(currency_id, "5000", "Expenses", "expense", "debit")
            .await;
        let re = self
            .create_account(currency_id, "3100", "Retained Earnings", "equity", "credit")
            .await;

        (
            id(&cash),
            id(&revenue),
            id(&expense),
            id(&re),
        )
    }

    pub async fn create_period(&self, name: &str, start: &str, end: &str) -> Value {
        self.server
            .post("/api/v1/periods")
            .add_header(self.auth_name(), self.auth_value())
            .json(&json!({
                "name": name,
                "start_date": start,
                "end_date": end,
            }))
            .await
            .json::<Value>()
    }

    pub async fn create_test_period(&self) -> String {
        let resp = self
            .create_period("FY2026", "2026-01-01", "2026-12-31")
            .await;
        id(&resp)
    }

    /// Post a balanced journal entry with the given debit/credit lines.
    /// `lines` is a vec of (account_id, debit_amount, credit_amount).
    pub async fn post_entry(
        &self,
        date: &str,
        description: &str,
        lines: Vec<(&str, &str, &str)>,
    ) -> Value {
        let line_json: Vec<Value> = lines
            .iter()
            .map(|(acct, dr, cr)| {
                let mut m = json!({"account_id": acct});
                if !dr.is_empty() && *dr != "0" {
                    m["debit_amount"] = json!(dr);
                }
                if !cr.is_empty() && *cr != "0" {
                    m["credit_amount"] = json!(cr);
                }
                m
            })
            .collect();

        self.server
            .post("/api/v1/journal-entries")
            .add_header(self.auth_name(), self.auth_value())
            .json(&json!({
                "entry_date": date,
                "description": description,
                "lines": line_json,
            }))
            .await
            .json::<Value>()
    }

    pub async fn set_retained_earnings(&self, account_id: &str) -> Value {
        self.server
            .patch("/api/v1/settings")
            .add_header(self.auth_name(), self.auth_value())
            .json(&json!({
                "retained_earnings_account_id": account_id,
            }))
            .await
            .json::<Value>()
    }

    pub async fn get_balance(&self, account_id: &str) -> Value {
        self.server
            .get(&format!("/api/v1/accounts/{account_id}/balance"))
            .add_header(self.auth_name(), self.auth_value())
            .await
            .json::<Value>()
    }

    /// Make an authenticated GET request.
    pub async fn get(&self, path: &str) -> Value {
        self.server
            .get(path)
            .add_header(self.auth_name(), self.auth_value())
            .await
            .json::<Value>()
    }

    /// Make an authenticated POST request with a JSON body.
    pub async fn post(&self, path: &str, body: &Value) -> Value {
        self.server
            .post(path)
            .add_header(self.auth_name(), self.auth_value())
            .json(body)
            .await
            .json::<Value>()
    }

    /// Make an authenticated PATCH request with a JSON body.
    pub async fn patch(&self, path: &str, body: &Value) -> Value {
        self.server
            .patch(path)
            .add_header(self.auth_name(), self.auth_value())
            .json(body)
            .await
            .json::<Value>()
    }
}

/// Extract the id string from a `{"data": {"id": "..."}}` response.
pub fn id(resp: &Value) -> String {
    resp["data"]["id"].as_str().unwrap().to_string()
}
