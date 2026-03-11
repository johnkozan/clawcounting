mod common;

use serde_json::json;

#[tokio::test]
async fn create_and_list_periods() {
    let app = common::TestApp::new().await;

    let resp = app.create_period("FY2026", "2026-01-01", "2026-12-31").await;
    let period = &resp["data"];
    assert_eq!(period["name"], "FY2026");
    assert_eq!(period["start_date"], "2026-01-01");
    assert_eq!(period["end_date"], "2026-12-31");
    assert!(period["closed_at"].is_null());

    let list = app
        .server
        .get("/api/v1/periods")
        .await
        .json::<serde_json::Value>();
    assert_eq!(list["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn overlapping_period_rejected() {
    let app = common::TestApp::new().await;

    app.create_period("FY2026", "2026-01-01", "2026-12-31").await;

    // Fully overlapping
    let resp = app
        .server
        .post("/api/v1/periods")
        .json(&json!({
            "name": "H1-2026",
            "start_date": "2026-01-01",
            "end_date": "2026-06-30"
        }))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    assert!(body["message"].as_str().unwrap().contains("overlap"));
}

#[tokio::test]
async fn non_overlapping_period_accepted() {
    let app = common::TestApp::new().await;

    app.create_period("FY2025", "2025-01-01", "2025-12-31").await;
    let resp = app.create_period("FY2026", "2026-01-01", "2026-12-31").await;
    assert_eq!(resp["data"]["name"], "FY2026");
}

#[tokio::test]
async fn start_after_end_rejected() {
    let app = common::TestApp::new().await;

    let resp = app
        .server
        .post("/api/v1/periods")
        .json(&json!({
            "name": "Bad",
            "start_date": "2026-12-31",
            "end_date": "2026-01-01"
        }))
        .await;
    resp.assert_status_bad_request();
}

#[tokio::test]
async fn close_period_and_reject_new_entries() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (cash, revenue, _, re) = app.create_test_accounts(&usd_id).await;
    let _period_id = app.create_test_period().await;

    // Post an entry
    app.post_entry(
        "2026-06-01",
        "Revenue",
        vec![(&cash, "100.00", ""), (&revenue, "", "100.00")],
    )
    .await;

    // Set retained earnings
    app.set_retained_earnings(&re).await;

    // Preview close
    let preview_resp = app
        .server
        .post(&format!("/api/v1/periods/{_period_id}/close?preview=true"))
        .await
        .json::<serde_json::Value>();
    assert_eq!(preview_resp["data"]["preview"], true);
    // Period should still be open after preview
    let period_after = app
        .server
        .get(&format!("/api/v1/periods/{_period_id}"))
        .await
        .json::<serde_json::Value>();
    assert!(period_after["data"]["closed_at"].is_null());

    // Actually close
    let close_resp = app
        .server
        .post(&format!("/api/v1/periods/{_period_id}/close"))
        .await
        .json::<serde_json::Value>();
    assert_eq!(close_resp["data"]["preview"], false);
    assert!(!close_resp["data"]["period"]["closed_at"].is_null());

    // Attempt to post entry in closed period
    let fail = app
        .server
        .post("/api/v1/journal-entries")
        .json(&json!({
            "entry_date": "2026-07-01",
            "description": "Should fail",
            "lines": [
                {"account_id": cash, "debit_amount": "10.00"},
                {"account_id": revenue, "credit_amount": "10.00"}
            ]
        }))
        .await;
    fail.assert_status_bad_request();
    let body = fail.json::<serde_json::Value>();
    assert!(body["message"].as_str().unwrap().contains("No open financial period"));
}

#[tokio::test]
async fn double_close_rejected() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;
    let (_, _, _, re) = app.create_test_accounts(&usd_id).await;
    let period_id = app.create_test_period().await;
    app.set_retained_earnings(&re).await;

    // First close succeeds
    app.server
        .post(&format!("/api/v1/periods/{period_id}/close"))
        .await
        .assert_status_ok();

    // Second close fails
    let resp = app
        .server
        .post(&format!("/api/v1/periods/{period_id}/close"))
        .await;
    resp.assert_status(axum::http::StatusCode::CONFLICT);
}

#[tokio::test]
async fn close_without_retained_earnings_fails() {
    let app = common::TestApp::new().await;
    let period_id = app.create_test_period().await;

    let resp = app
        .server
        .post(&format!("/api/v1/periods/{period_id}/close"))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("Retained earnings"));
}
