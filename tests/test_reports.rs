mod common;

use serde_json::Value;

/// Setup: create USD, accounts (Cash, AR, Revenue, Expense, Retained Earnings),
/// period FY2026, set retained earnings, and post journal entries.
///
/// Entries:
///   1. DR Cash 1000, CR Revenue 1000 (revenue earned)
///   2. DR Expenses 500, CR Cash 500 (expense paid)
///   3. DR AR 200, CR Revenue 200 (revenue on credit)
///
/// Expected balances:
///   Cash:     DR=1000, CR=500  → net debit 500
///   AR:       DR=200,  CR=0    → net debit 200
///   Revenue:  DR=0,    CR=1200 → net credit 1200
///   Expenses: DR=500,  CR=0    → net debit 500
///   RE:       DR=0,    CR=0    → net 0
async fn setup_test_data(app: &common::TestApp) -> TestData {
    let usd_id = app.create_usd().await;
    let (cash_id, revenue_id, expense_id, re_id) =
        app.create_test_accounts(&usd_id).await;

    // Create an additional asset account: Accounts Receivable
    let ar = app
        .create_account(&usd_id, "1100", "Accounts Receivable", "asset", "debit")
        .await;
    let ar_id = common::id(&ar);

    let period = app.create_period("FY2026", "2026-01-01", "2026-12-31").await;
    let period_id = common::id(&period);

    app.set_retained_earnings(&re_id).await;

    // Entry 1: DR Cash 1000, CR Revenue 1000
    app.post_entry(
        "2026-03-15",
        "Revenue earned",
        vec![(&cash_id, "1000.00", "0"), (&revenue_id, "0", "1000.00")],
    )
    .await;

    // Entry 2: DR Expenses 500, CR Cash 500
    app.post_entry(
        "2026-04-01",
        "Rent payment",
        vec![(&expense_id, "500.00", "0"), (&cash_id, "0", "500.00")],
    )
    .await;

    // Entry 3: DR AR 200, CR Revenue 200
    app.post_entry(
        "2026-06-15",
        "Credit sale",
        vec![(&ar_id, "200.00", "0"), (&revenue_id, "0", "200.00")],
    )
    .await;

    TestData {
        usd_id,
        cash_id,
        ar_id,
        revenue_id,
        expense_id,
        re_id,
        period_id,
    }
}

#[allow(dead_code)]
struct TestData {
    usd_id: String,
    cash_id: String,
    ar_id: String,
    revenue_id: String,
    expense_id: String,
    re_id: String,
    period_id: String,
}

// ── Trial Balance ──────────────────────────────────────────────

#[tokio::test]
async fn test_trial_balance_all_time() {
    let app = common::TestApp::new().await;
    let _data = setup_test_data(&app).await;

    let resp: Value = app
        .server
        .get("/api/v1/reports/trial-balance")
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];
    assert_eq!(report["is_balanced"], true);

    let rows = report["rows"].as_array().unwrap();
    // 4 accounts with activity: Cash, AR, Revenue, Expenses
    // RE has no entries so no balance rows
    assert_eq!(rows.len(), 4);

    // Verify grand totals balance
    let grand_dr = &report["display_grand_total_debits"];
    let grand_cr = &report["display_grand_total_credits"];
    assert_eq!(grand_dr, grand_cr);

    // Grand total debits = 1000 + 500 + 200 = 1700
    assert_eq!(report["display_grand_total_debits"], "1700.00");
    assert_eq!(report["display_grand_total_credits"], "1700.00");

    // Verify individual accounts are present and correct
    let cash_row = rows.iter().find(|r| r["account_number"] == "1000").unwrap();
    assert_eq!(cash_row["display_debit_total"], "1000.00");
    assert_eq!(cash_row["display_credit_total"], "500.00");

    let revenue_row = rows.iter().find(|r| r["account_number"] == "4000").unwrap();
    assert_eq!(revenue_row["display_debit_total"], "0.00");
    assert_eq!(revenue_row["display_credit_total"], "1200.00");
}

#[tokio::test]
async fn test_trial_balance_by_period() {
    let app = common::TestApp::new().await;
    let data = setup_test_data(&app).await;

    let resp: Value = app
        .server
        .get(&format!(
            "/api/v1/reports/trial-balance?period_id={}",
            data.period_id
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];
    assert_eq!(report["is_balanced"], true);
    assert_eq!(report["period_id"], data.period_id);
}

#[tokio::test]
async fn test_trial_balance_by_currency() {
    let app = common::TestApp::new().await;
    let data = setup_test_data(&app).await;

    let resp: Value = app
        .server
        .get(&format!(
            "/api/v1/reports/trial-balance?currency_id={}",
            data.usd_id
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];
    assert_eq!(report["is_balanced"], true);
    assert_eq!(report["currency_id"], data.usd_id);
    // 4 accounts with activity (RE has no entries)
    assert_eq!(report["rows"].as_array().unwrap().len(), 4);
}

#[tokio::test]
async fn test_trial_balance_empty() {
    let app = common::TestApp::new().await;

    let resp: Value = app
        .server
        .get("/api/v1/reports/trial-balance")
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];
    assert_eq!(report["is_balanced"], true);
    assert_eq!(report["rows"].as_array().unwrap().len(), 0);
}

// ── Balance Sheet ──────────────────────────────────────────────

#[tokio::test]
async fn test_balance_sheet_current() {
    let app = common::TestApp::new().await;
    let _data = setup_test_data(&app).await;

    let resp: Value = app
        .server
        .get("/api/v1/reports/balance-sheet")
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];

    // Assets: Cash(500) + AR(200) = 700
    assert_eq!(report["assets"]["display_total"], "700.00");
    let asset_accounts = report["assets"]["accounts"].as_array().unwrap();
    assert_eq!(asset_accounts.len(), 2);

    // Liabilities: none
    assert_eq!(report["liabilities"]["display_total"], "0.00");

    // Equity: RE has 0 balance (period not closed)
    assert_eq!(report["equity"]["display_total"], "0.00");

    // Before period close, balance sheet won't balance because
    // revenue/expense aren't included and haven't been closed to RE
    // total_assets = 700, total_l_and_e = 0
    // This is expected — the balance sheet balances only after period close
}

#[tokio::test]
async fn test_balance_sheet_after_period_close() {
    let app = common::TestApp::new().await;
    let data = setup_test_data(&app).await;

    // Close the period
    app.server
        .post(&format!("/api/v1/periods/{}/close", data.period_id))
        .add_header(app.auth_name(), app.auth_value())
        .await;

    let resp: Value = app
        .server
        .get("/api/v1/reports/balance-sheet")
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];

    // After close: revenue/expense zeroed, net income -> RE
    // Net income = 1200 - 500 = 700
    // RE now = 700
    // Assets: Cash(500) + AR(200) = 700
    // Equity: RE(700) = 700
    assert_eq!(report["display_total_assets"], "700.00");
    assert_eq!(report["display_total_liabilities_and_equity"], "700.00");
    assert_eq!(report["is_balanced"], true);
}

#[tokio::test]
async fn test_balance_sheet_as_of_date() {
    let app = common::TestApp::new().await;
    let _data = setup_test_data(&app).await;

    // As of 2026-03-31: only entry 1 (Cash DR 1000, Revenue CR 1000)
    let resp: Value = app
        .server
        .get("/api/v1/reports/balance-sheet?as_of_date=2026-03-31")
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];
    // Only cash should appear in assets
    assert_eq!(report["assets"]["display_total"], "1000.00");
    assert_eq!(report["as_of_date"], "2026-03-31");
}

#[tokio::test]
async fn test_balance_sheet_by_period() {
    let app = common::TestApp::new().await;
    let data = setup_test_data(&app).await;

    let resp: Value = app
        .server
        .get(&format!(
            "/api/v1/reports/balance-sheet?period_id={}",
            data.period_id
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];
    // Should include all balances through the period end
    assert_eq!(report["assets"]["display_total"], "700.00");
    assert_eq!(report["period_id"], data.period_id);
}

// ── Income Statement ──────────────────────────────────────────

#[tokio::test]
async fn test_income_statement() {
    let app = common::TestApp::new().await;
    let data = setup_test_data(&app).await;

    let resp: Value = app
        .server
        .get(&format!(
            "/api/v1/reports/income-statement?period_id={}",
            data.period_id
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];
    assert_eq!(report["period_id"], data.period_id);

    // Revenue: 1200
    assert_eq!(report["display_total_revenue"], "1200.00");
    let revenue = report["revenue"].as_array().unwrap();
    assert_eq!(revenue.len(), 1);
    assert_eq!(revenue[0]["display_amount"], "1200.00");

    // Expenses: 500
    assert_eq!(report["display_total_expenses"], "500.00");
    let expenses = report["expenses"].as_array().unwrap();
    assert_eq!(expenses.len(), 1);
    assert_eq!(expenses[0]["display_amount"], "500.00");

    // Net Income: 700
    assert_eq!(report["display_net_income"], "700.00");
}

#[tokio::test]
async fn test_income_statement_requires_period() {
    let app = common::TestApp::new().await;

    let resp = app
        .server
        .get("/api/v1/reports/income-statement")
        .add_header(app.auth_name(), app.auth_value())
        .await;

    assert_eq!(resp.status_code(), 400);
}

// ── General Ledger ─────────────────────────────────────────────

#[tokio::test]
async fn test_general_ledger_descending() {
    let app = common::TestApp::new().await;
    let data = setup_test_data(&app).await;

    let resp: Value = app
        .server
        .get(&format!(
            "/api/v1/reports/general-ledger?account_id={}",
            data.cash_id
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];
    assert_eq!(report["account_name"], "Cash");
    assert_eq!(report["normal_balance"], "debit");

    let lines = report["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 2); // Two entries touch cash

    // Default sort is descending, so most recent first
    // Line 1 (most recent): 2026-04-01 Rent payment (CR 500)
    // Line 2: 2026-03-15 Revenue earned (DR 1000)
    assert_eq!(lines[0]["entry_date"], "2026-04-01");
    assert_eq!(lines[1]["entry_date"], "2026-03-15");

    // Running balances (descending):
    // Starting balance = sum through end = 500 (net debit)
    assert_eq!(report["display_starting_balance"], "500.00");

    // First line (most recent, CR 500): balance after = 500
    assert_eq!(lines[0]["display_running_balance"], "500.00");
    // Second line (DR 1000): balance after = 1000
    assert_eq!(lines[1]["display_running_balance"], "1000.00");

    assert_eq!(report["has_more"], false);
}

#[tokio::test]
async fn test_general_ledger_ascending() {
    let app = common::TestApp::new().await;
    let data = setup_test_data(&app).await;

    let resp: Value = app
        .server
        .get(&format!(
            "/api/v1/reports/general-ledger?account_id={}&sort=asc",
            data.cash_id
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];
    let lines = report["lines"].as_array().unwrap();

    // Ascending: oldest first
    assert_eq!(lines[0]["entry_date"], "2026-03-15");
    assert_eq!(lines[1]["entry_date"], "2026-04-01");

    // Starting balance = 0 (nothing before range)
    assert_eq!(report["display_starting_balance"], "0.00");

    // Running balances (ascending):
    // After DR 1000: balance = 1000
    assert_eq!(lines[0]["display_running_balance"], "1000.00");
    // After CR 500: balance = 500
    assert_eq!(lines[1]["display_running_balance"], "500.00");
}

#[tokio::test]
async fn test_general_ledger_with_period_filter() {
    let app = common::TestApp::new().await;
    let data = setup_test_data(&app).await;

    let resp: Value = app
        .server
        .get(&format!(
            "/api/v1/reports/general-ledger?account_id={}&period_id={}&sort=asc",
            data.cash_id, data.period_id
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];
    let lines = report["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 2);
}

#[tokio::test]
async fn test_general_ledger_with_date_range() {
    let app = common::TestApp::new().await;
    let data = setup_test_data(&app).await;

    // Only entries up to 2026-03-31 — should only include the first cash entry
    let resp: Value = app
        .server
        .get(&format!(
            "/api/v1/reports/general-ledger?account_id={}&start_date=2026-01-01&end_date=2026-03-31&sort=asc",
            data.cash_id
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];
    let lines = report["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["entry_date"], "2026-03-15");
    assert_eq!(lines[0]["display_running_balance"], "1000.00");
}

#[tokio::test]
async fn test_general_ledger_requires_account() {
    let app = common::TestApp::new().await;

    let resp = app
        .server
        .get("/api/v1/reports/general-ledger")
        .add_header(app.auth_name(), app.auth_value())
        .await;

    assert_eq!(resp.status_code(), 400);
}

#[tokio::test]
async fn test_general_ledger_revenue_account() {
    let app = common::TestApp::new().await;
    let data = setup_test_data(&app).await;

    let resp: Value = app
        .server
        .get(&format!(
            "/api/v1/reports/general-ledger?account_id={}&sort=asc",
            data.revenue_id
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];
    assert_eq!(report["normal_balance"], "credit");

    let lines = report["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 2); // Two revenue entries

    // Revenue is credit-normal, so net = credits - debits
    // Starting balance = 0
    // After CR 1000: net = 1000
    assert_eq!(lines[0]["display_running_balance"], "1000.00");
    // After CR 200: net = 1200
    assert_eq!(lines[1]["display_running_balance"], "1200.00");
}

#[tokio::test]
async fn test_general_ledger_pagination() {
    let app = common::TestApp::new().await;
    let data = setup_test_data(&app).await;

    // Post more entries to cash to test pagination
    for i in 0..5 {
        app.post_entry(
            &format!("2026-07-{:02}", i + 1),
            &format!("Payment {}", i + 1),
            vec![
                (&data.cash_id, "100.00", "0"),
                (&data.revenue_id, "0", "100.00"),
            ],
        )
        .await;
    }

    // Cash now has 7 entries. Fetch with limit=3 ascending
    let resp: Value = app
        .server
        .get(&format!(
            "/api/v1/reports/general-ledger?account_id={}&sort=asc&limit=3",
            data.cash_id
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report = &resp["data"];
    let lines = report["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 3);
    assert_eq!(report["has_more"], true);
    assert!(report["next_cursor"].is_string());

    let cursor = report["next_cursor"].as_str().unwrap();

    // Fetch page 2
    let resp2: Value = app
        .server
        .get(&format!(
            "/api/v1/reports/general-ledger?account_id={}&sort=asc&limit=3&cursor={}",
            data.cash_id, cursor
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report2 = &resp2["data"];
    let lines2 = report2["lines"].as_array().unwrap();
    assert_eq!(lines2.len(), 3);
    assert_eq!(report2["has_more"], true);

    // The running balance should continue from page 1
    // Page 1 last balance should be the starting point for page 2 lines
    let page1_last_balance: i128 = lines[2]["running_balance"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let page2_starting: i128 = report2["starting_balance"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(page1_last_balance, page2_starting);

    // Fetch page 3
    let cursor2 = report2["next_cursor"].as_str().unwrap();
    let resp3: Value = app
        .server
        .get(&format!(
            "/api/v1/reports/general-ledger?account_id={}&sort=asc&limit=3&cursor={}",
            data.cash_id, cursor2
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let report3 = &resp3["data"];
    let lines3 = report3["lines"].as_array().unwrap();
    assert_eq!(lines3.len(), 1); // Only 1 remaining
    assert_eq!(report3["has_more"], false);
}

// ── Cross-Report Consistency ──────────────────────────────────

#[tokio::test]
async fn test_trial_balance_matches_individual_balances() {
    let app = common::TestApp::new().await;
    let _data = setup_test_data(&app).await;

    // Get trial balance
    let tb_resp: Value = app
        .server
        .get("/api/v1/reports/trial-balance")
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    // Verify each account's balance matches the individual balance endpoint
    let rows = tb_resp["data"]["rows"].as_array().unwrap();
    for row in rows {
        let account_id = row["account_id"].as_str().unwrap();
        let bal_resp: Value = app.get_balance(account_id).await;
        let bal = &bal_resp["data"];

        assert_eq!(
            row["debit_total"],
            bal["total_debits"],
            "Debit mismatch for account {}",
            account_id
        );
        assert_eq!(
            row["credit_total"],
            bal["total_credits"],
            "Credit mismatch for account {}",
            account_id
        );
    }
}

#[tokio::test]
async fn test_income_statement_matches_trial_balance() {
    let app = common::TestApp::new().await;
    let data = setup_test_data(&app).await;

    let is_resp: Value = app
        .server
        .get(&format!(
            "/api/v1/reports/income-statement?period_id={}",
            data.period_id
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    let tb_resp: Value = app
        .server
        .get(&format!(
            "/api/v1/reports/trial-balance?period_id={}",
            data.period_id
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json();

    // Revenue from income statement should match trial balance
    let is_revenue = &is_resp["data"]["display_total_revenue"];
    let tb_rows = tb_resp["data"]["rows"].as_array().unwrap();
    let tb_revenue = tb_rows
        .iter()
        .find(|r| r["account_type"] == "revenue")
        .unwrap();
    assert_eq!(is_revenue, &tb_revenue["display_credit_total"]);
}
