mod common;

use serde_json::json;

#[tokio::test]
async fn create_balanced_entry() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, revenue, _, _) = app.create_test_accounts(&usd_id).await;
    app.create_test_period().await;

    let resp = app
        .post_entry(
            "2026-03-15",
            "Service revenue",
            vec![(&cash, "250.00", ""), (&revenue, "", "250.00")],
        )
        .await;

    let entry = &resp["data"];
    assert_eq!(entry["description"], "Service revenue");
    assert_eq!(entry["entry_date"], "2026-03-15");
    assert_eq!(entry["is_reversal"], false);
    assert_eq!(entry["lines"].as_array().unwrap().len(), 2);

    // Verify display amounts
    let lines = entry["lines"].as_array().unwrap();
    let debit_line = lines.iter().find(|l| l["debit_amount"] != "0").unwrap();
    assert_eq!(debit_line["display_debit"], "250.00");
    let credit_line = lines.iter().find(|l| l["credit_amount"] != "0").unwrap();
    assert_eq!(credit_line["display_credit"], "250.00");
}

#[tokio::test]
async fn unbalanced_entry_rejected() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, revenue, _, _) = app.create_test_accounts(&usd_id).await;
    app.create_test_period().await;

    let resp = app
        .server
        .post("/api/v1/journal-entries")
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({
            "entry_date": "2026-03-15",
            "description": "Unbalanced",
            "lines": [
                {"account_id": cash, "debit_amount": "100.00"},
                {"account_id": revenue, "credit_amount": "50.00"}
            ]
        }))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    assert_eq!(body["code"], "UNBALANCED_ENTRY");
    assert!(body["message"].as_str().unwrap().contains("100.00"));
    assert!(body["message"].as_str().unwrap().contains("50.00"));
}

#[tokio::test]
async fn single_line_entry_rejected() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, _, _, _) = app.create_test_accounts(&usd_id).await;
    app.create_test_period().await;

    let resp = app
        .server
        .post("/api/v1/journal-entries")
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({
            "entry_date": "2026-03-15",
            "description": "One line",
            "lines": [
                {"account_id": cash, "debit_amount": "100.00"}
            ]
        }))
        .await;
    resp.assert_status_bad_request();
}

#[tokio::test]
async fn posting_to_control_account_rejected() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (_, revenue, _, _) = app.create_test_accounts(&usd_id).await;
    app.create_test_period().await;

    // Create a control account with subledger
    let control = app
        .server
        .post("/api/v1/accounts")
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({
            "currency_id": usd_id,
            "account_number": "1200",
            "name": "AR Control",
            "account_type": "asset",
            "normal_balance": "debit",
            "has_subledger": true
        }))
        .await
        .json::<serde_json::Value>();
    let control_id = control["data"]["id"].as_str().unwrap();

    let resp = app
        .server
        .post("/api/v1/journal-entries")
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({
            "entry_date": "2026-03-15",
            "description": "To control",
            "lines": [
                {"account_id": control_id, "debit_amount": "100.00"},
                {"account_id": revenue, "credit_amount": "100.00"}
            ]
        }))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    assert!(body["message"].as_str().unwrap().contains("control account"));
}

#[tokio::test]
async fn posting_to_inactive_account_rejected() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, revenue, _, _) = app.create_test_accounts(&usd_id).await;
    app.create_test_period().await;

    // Deactivate the cash account
    app.server
        .patch(&format!("/api/v1/accounts/{cash}"))
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({"is_active": false}))
        .await;

    let resp = app
        .server
        .post("/api/v1/journal-entries")
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({
            "entry_date": "2026-03-15",
            "description": "To inactive",
            "lines": [
                {"account_id": cash, "debit_amount": "100.00"},
                {"account_id": revenue, "credit_amount": "100.00"}
            ]
        }))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    assert!(body["message"].as_str().unwrap().contains("inactive"));
}

#[tokio::test]
async fn entry_without_open_period_rejected() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, revenue, _, _) = app.create_test_accounts(&usd_id).await;
    // No period created

    let resp = app
        .server
        .post("/api/v1/journal-entries")
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({
            "entry_date": "2026-03-15",
            "description": "No period",
            "lines": [
                {"account_id": cash, "debit_amount": "100.00"},
                {"account_id": revenue, "credit_amount": "100.00"}
            ]
        }))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    assert!(body["message"].as_str().unwrap().contains("No open financial period"));
}

#[tokio::test]
async fn list_journal_entries_with_period_filter() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, revenue, _, _) = app.create_test_accounts(&usd_id).await;
    let period_id = app.create_test_period().await;

    app.post_entry(
        "2026-03-01",
        "Entry 1",
        vec![(&cash, "100.00", ""), (&revenue, "", "100.00")],
    )
    .await;
    app.post_entry(
        "2026-04-01",
        "Entry 2",
        vec![(&cash, "200.00", ""), (&revenue, "", "200.00")],
    )
    .await;

    let resp = app
        .server
        .get(&format!(
            "/api/v1/journal-entries?period_id={period_id}"
        ))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json::<serde_json::Value>();
    assert_eq!(resp["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn get_journal_entry_with_lines() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, revenue, _, _) = app.create_test_accounts(&usd_id).await;
    app.create_test_period().await;

    let create_resp = app
        .post_entry(
            "2026-03-01",
            "Test entry",
            vec![(&cash, "500.00", ""), (&revenue, "", "500.00")],
        )
        .await;
    let entry_id = create_resp["data"]["id"].as_str().unwrap();

    let get_resp = app
        .server
        .get(&format!("/api/v1/journal-entries/{entry_id}"))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json::<serde_json::Value>();

    assert_eq!(get_resp["data"]["description"], "Test entry");
    assert_eq!(get_resp["data"]["lines"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn reverse_journal_entry() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, revenue, _, _) = app.create_test_accounts(&usd_id).await;
    app.create_test_period().await;

    // Create original entry
    let original = app
        .post_entry(
            "2026-03-01",
            "Original",
            vec![(&cash, "300.00", ""), (&revenue, "", "300.00")],
        )
        .await;
    let original_id = original["data"]["id"].as_str().unwrap();

    // Reverse it
    let reversal = app
        .server
        .post(&format!("/api/v1/journal-entries/{original_id}/reverse"))
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({"entry_date": "2026-03-15"}))
        .await
        .json::<serde_json::Value>();

    assert_eq!(reversal["data"]["is_reversal"], true);
    assert_eq!(reversal["data"]["reverses_id"], original_id);
    assert_eq!(reversal["data"]["entry_date"], "2026-03-15");

    // Verify debits/credits are swapped
    let rev_lines = reversal["data"]["lines"].as_array().unwrap();
    let orig_lines = original["data"]["lines"].as_array().unwrap();

    // The debit line in original should be credit in reversal (and vice versa)
    for orig_line in orig_lines {
        let matching = rev_lines
            .iter()
            .find(|l| l["account_id"] == orig_line["account_id"])
            .unwrap();
        assert_eq!(matching["debit_amount"], orig_line["credit_amount"]);
        assert_eq!(matching["credit_amount"], orig_line["debit_amount"]);
    }

    // Check balance is back to zero
    let cash_balance = app.get_balance(&cash).await;
    assert_eq!(cash_balance["data"]["net_balance"], "0");
}

#[tokio::test]
async fn reversing_a_reversal_rejected() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, revenue, _, _) = app.create_test_accounts(&usd_id).await;
    app.create_test_period().await;

    let original = app
        .post_entry(
            "2026-03-01",
            "Original",
            vec![(&cash, "100.00", ""), (&revenue, "", "100.00")],
        )
        .await;
    let original_id = original["data"]["id"].as_str().unwrap();

    // Reverse
    let reversal = app
        .server
        .post(&format!("/api/v1/journal-entries/{original_id}/reverse"))
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({}))
        .await
        .json::<serde_json::Value>();
    let reversal_id = reversal["data"]["id"].as_str().unwrap();

    // Try to reverse the reversal
    let resp = app
        .server
        .post(&format!("/api/v1/journal-entries/{reversal_id}/reverse"))
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({}))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    assert!(body["message"].as_str().unwrap().contains("Cannot reverse a reversal"));
}

#[tokio::test]
async fn double_reversal_rejected() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, revenue, _, _) = app.create_test_accounts(&usd_id).await;
    app.create_test_period().await;

    let original = app
        .post_entry(
            "2026-03-01",
            "Original",
            vec![(&cash, "100.00", ""), (&revenue, "", "100.00")],
        )
        .await;
    let original_id = original["data"]["id"].as_str().unwrap();

    // First reversal succeeds
    app.server
        .post(&format!("/api/v1/journal-entries/{original_id}/reverse"))
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({}))
        .await
        .assert_status_ok();

    // Second reversal of same entry fails
    let resp = app
        .server
        .post(&format!("/api/v1/journal-entries/{original_id}/reverse"))
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({}))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    assert!(body["message"].as_str().unwrap().contains("already been reversed"));
}

#[tokio::test]
async fn balance_updates_after_entries() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, revenue, expense, _) = app.create_test_accounts(&usd_id).await;
    app.create_test_period().await;

    // Post revenue: DR Cash 500, CR Revenue 500
    app.post_entry(
        "2026-03-01",
        "Revenue",
        vec![(&cash, "500.00", ""), (&revenue, "", "500.00")],
    )
    .await;

    // Post expense: DR Expense 200, CR Cash 200
    app.post_entry(
        "2026-03-02",
        "Expense",
        vec![(&expense, "200.00", ""), (&cash, "", "200.00")],
    )
    .await;

    // Check cash balance: 500 debit - 200 credit = 300
    let cash_bal = app.get_balance(&cash).await;
    assert_eq!(cash_bal["data"]["display_balance"], "300.00");
    assert_eq!(cash_bal["data"]["display_debits"], "500.00");
    assert_eq!(cash_bal["data"]["display_credits"], "200.00");

    // Check revenue balance: 500 credit (credit-normal → net = 500)
    let rev_bal = app.get_balance(&revenue).await;
    assert_eq!(rev_bal["data"]["display_balance"], "500.00");

    // Check expense balance: 200 debit (debit-normal → net = 200)
    let exp_bal = app.get_balance(&expense).await;
    assert_eq!(exp_bal["data"]["display_balance"], "200.00");
}

#[tokio::test]
async fn account_transactions_endpoint() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, revenue, _, _) = app.create_test_accounts(&usd_id).await;
    app.create_test_period().await;

    app.post_entry(
        "2026-03-01",
        "Entry 1",
        vec![(&cash, "100.00", ""), (&revenue, "", "100.00")],
    )
    .await;
    app.post_entry(
        "2026-04-01",
        "Entry 2",
        vec![(&cash, "200.00", ""), (&revenue, "", "200.00")],
    )
    .await;

    let txns = app
        .server
        .get(&format!("/api/v1/accounts/{cash}/transactions"))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json::<serde_json::Value>();

    let data = txns["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0]["display_debit"], "100.00");
    assert_eq!(data[1]["display_debit"], "200.00");
}

#[tokio::test]
async fn multi_line_compound_entry() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, revenue, expense, _) = app.create_test_accounts(&usd_id).await;
    app.create_test_period().await;

    // 3-line compound entry: DR Cash 100, CR Revenue 70, CR Expense 30 (refund)
    // Wait, that's debit cash and credit two accounts. Let me do:
    // DR Cash 100, DR Expense 50, CR Revenue 150
    let resp = app
        .post_entry(
            "2026-06-01",
            "Compound entry",
            vec![
                (&cash, "100.00", ""),
                (&expense, "50.00", ""),
                (&revenue, "", "150.00"),
            ],
        )
        .await;

    assert_eq!(resp["data"]["lines"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn entry_with_metadata() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, revenue, _, _) = app.create_test_accounts(&usd_id).await;
    app.create_test_period().await;

    let resp = app
        .server
        .post("/api/v1/journal-entries")
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({
            "entry_date": "2026-03-15",
            "description": "With metadata",
            "reference": "INV-001",
            "metadata": {"invoice_id": "inv_123", "customer": "Acme"},
            "lines": [
                {"account_id": cash, "debit_amount": "100.00"},
                {"account_id": revenue, "credit_amount": "100.00"}
            ]
        }))
        .await
        .json::<serde_json::Value>();

    assert_eq!(resp["data"]["reference"], "INV-001");
    assert_eq!(resp["data"]["metadata"]["invoice_id"], "inv_123");
}
