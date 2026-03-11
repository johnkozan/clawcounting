mod common;

use serde_json::json;

#[tokio::test]
async fn create_and_get_account() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;

    let resp = app
        .create_account(&usd_id, "1000", "Cash", "asset", "debit")
        .await;
    let acct = &resp["data"];
    assert_eq!(acct["name"], "Cash");
    assert_eq!(acct["account_number"], "1000");
    assert_eq!(acct["account_type"], "asset");
    assert_eq!(acct["normal_balance"], "debit");
    assert_eq!(acct["is_active"], true);

    let id = acct["id"].as_str().unwrap();
    let get_resp = app
        .server
        .get(&format!("/api/v1/accounts/{id}"))
        .await
        .json::<serde_json::Value>();
    assert_eq!(get_resp["data"]["name"], "Cash");
}

#[tokio::test]
async fn list_accounts_with_type_filter() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    app.create_test_accounts(&usd_id).await;

    // All accounts
    let resp = app
        .server
        .get("/api/v1/accounts")
        .await
        .json::<serde_json::Value>();
    assert_eq!(resp["data"].as_array().unwrap().len(), 4);

    // Filter by type
    let resp = app
        .server
        .get("/api/v1/accounts?account_type=asset")
        .await
        .json::<serde_json::Value>();
    assert_eq!(resp["data"].as_array().unwrap().len(), 1);
    assert_eq!(resp["data"][0]["account_type"], "asset");
}

#[tokio::test]
async fn duplicate_account_number_rejected() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;

    app.create_account(&usd_id, "1000", "Cash", "asset", "debit")
        .await;

    let resp = app
        .server
        .post("/api/v1/accounts")
        .json(&json!({
            "currency_id": usd_id,
            "account_number": "1000",
            "name": "Another Cash",
            "account_type": "asset",
            "normal_balance": "debit"
        }))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    assert!(body["message"].as_str().unwrap().contains("already exists"));
}

#[tokio::test]
async fn invalid_account_type_rejected() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;

    let resp = app
        .server
        .post("/api/v1/accounts")
        .json(&json!({
            "currency_id": usd_id,
            "account_number": "9999",
            "name": "Bad",
            "account_type": "invalid",
            "normal_balance": "debit"
        }))
        .await;
    resp.assert_status_bad_request();
}

#[tokio::test]
async fn subledger_create_sub_account() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;

    // Create parent control account with subledger
    let parent = app
        .server
        .post("/api/v1/accounts")
        .json(&json!({
            "currency_id": usd_id,
            "account_number": "1200",
            "name": "Accounts Receivable",
            "account_type": "asset",
            "normal_balance": "debit",
            "has_subledger": true
        }))
        .await
        .json::<serde_json::Value>();
    let parent_id = parent["data"]["id"].as_str().unwrap();

    // Create sub-account (inherits type, currency, normal_balance from parent)
    let sub = app
        .server
        .post("/api/v1/accounts")
        .json(&json!({
            "account_number": "1200-001",
            "name": "AR - Customer A",
            "parent_id": parent_id,
            "entity_id": "customer-a"
        }))
        .await
        .json::<serde_json::Value>();

    assert_eq!(sub["data"]["parent_id"], parent_id);
    assert_eq!(sub["data"]["entity_id"], "customer-a");
    assert_eq!(sub["data"]["account_type"], "asset");
    assert_eq!(sub["data"]["normal_balance"], "debit");
    assert_eq!(sub["data"]["currency_id"], usd_id);

    // List sub-accounts
    let subs = app
        .server
        .get(&format!("/api/v1/accounts/{parent_id}/sub-accounts"))
        .await
        .json::<serde_json::Value>();
    assert_eq!(subs["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn sub_account_without_entity_id_rejected() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;

    let parent = app
        .server
        .post("/api/v1/accounts")
        .json(&json!({
            "currency_id": usd_id,
            "account_number": "1200",
            "name": "AR",
            "account_type": "asset",
            "normal_balance": "debit",
            "has_subledger": true
        }))
        .await
        .json::<serde_json::Value>();
    let parent_id = parent["data"]["id"].as_str().unwrap();

    let resp = app
        .server
        .post("/api/v1/accounts")
        .json(&json!({
            "account_number": "1200-001",
            "name": "Missing Entity",
            "parent_id": parent_id,
        }))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    assert!(body["message"].as_str().unwrap().contains("entity_id"));
}

#[tokio::test]
async fn sub_account_on_non_subledger_parent_rejected() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;

    let parent = app
        .create_account(&usd_id, "1000", "Cash", "asset", "debit")
        .await;
    let parent_id = parent["data"]["id"].as_str().unwrap();

    let resp = app
        .server
        .post("/api/v1/accounts")
        .json(&json!({
            "account_number": "1000-001",
            "name": "Sub Cash",
            "parent_id": parent_id,
            "entity_id": "sub-1"
        }))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    assert!(body["message"].as_str().unwrap().contains("subledger"));
}

#[tokio::test]
async fn update_account_deactivate() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;

    let acct = app
        .create_account(&usd_id, "1000", "Cash", "asset", "debit")
        .await;
    let acct_id = acct["data"]["id"].as_str().unwrap();

    let resp = app
        .server
        .patch(&format!("/api/v1/accounts/{acct_id}"))
        .json(&json!({"is_active": false}))
        .await
        .json::<serde_json::Value>();
    assert_eq!(resp["data"]["is_active"], false);
}
