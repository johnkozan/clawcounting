mod common;

use serde_json::json;

#[tokio::test]
async fn create_and_get_currency() {
    let app = common::TestApp::new().await;

    let resp = app
        .create_currency("USD", "US Dollar", "$", 2, "fiat")
        .await;
    let currency = &resp["data"];
    assert_eq!(currency["code"], "USD");
    assert_eq!(currency["name"], "US Dollar");
    assert_eq!(currency["symbol"], "$");
    assert_eq!(currency["asset_scale"], 2);
    assert_eq!(currency["asset_type"], "fiat");
    assert_eq!(currency["caip19_id"], "swift:0/iso4217:USD");

    // GET by id
    let id = currency["id"].as_str().unwrap();
    let get_resp = app
        .server
        .get(&format!("/api/v1/currencies/{id}"))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json::<serde_json::Value>();
    assert_eq!(get_resp["data"]["code"], "USD");
}

#[tokio::test]
async fn list_currencies_with_pagination() {
    let app = common::TestApp::new().await;

    app.create_currency("USD", "US Dollar", "$", 2, "fiat")
        .await;
    app.create_currency("EUR", "Euro", "€", 2, "fiat").await;
    app.create_currency("GBP", "British Pound", "£", 2, "fiat")
        .await;

    // List all
    let resp = app
        .server
        .get("/api/v1/currencies")
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json::<serde_json::Value>();
    assert_eq!(resp["data"].as_array().unwrap().len(), 3);
    assert_eq!(resp["has_more"], false);

    // Paginate with limit=2
    let resp = app
        .server
        .get("/api/v1/currencies?limit=2")
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json::<serde_json::Value>();
    assert_eq!(resp["data"].as_array().unwrap().len(), 2);
    assert_eq!(resp["has_more"], true);
    let cursor = resp["next_cursor"].as_str().unwrap();

    // Get next page
    let resp = app
        .server
        .get(&format!("/api/v1/currencies?limit=2&cursor={cursor}"))
        .add_header(app.auth_name(), app.auth_value())
        .await
        .json::<serde_json::Value>();
    assert_eq!(resp["data"].as_array().unwrap().len(), 1);
    assert_eq!(resp["has_more"], false);
}

#[tokio::test]
async fn duplicate_currency_code_rejected() {
    let app = common::TestApp::new().await;

    app.create_currency("USD", "US Dollar", "$", 2, "fiat")
        .await;

    let resp = app
        .server
        .post("/api/v1/currencies")
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({
            "code": "USD",
            "name": "Another Dollar",
            "symbol": "$",
            "asset_scale": 2,
            "asset_type": "fiat",
            "caip19_id": "swift:0/iso4217:USD2"
        }))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    assert_eq!(body["code"], "VALIDATION_ERROR");
    assert!(body["message"].as_str().unwrap().contains("already exists"));
}

#[tokio::test]
async fn duplicate_caip19_rejected() {
    let app = common::TestApp::new().await;

    app.create_currency("USD", "US Dollar", "$", 2, "fiat")
        .await;

    let resp = app
        .server
        .post("/api/v1/currencies")
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({
            "code": "XYZ",
            "name": "Fake",
            "symbol": "?",
            "asset_scale": 2,
            "asset_type": "fiat",
            "caip19_id": "swift:0/iso4217:USD"
        }))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    assert!(body["message"].as_str().unwrap().contains("CAIP-19"));
}

#[tokio::test]
async fn invalid_asset_type_rejected() {
    let app = common::TestApp::new().await;

    let resp = app
        .server
        .post("/api/v1/currencies")
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({
            "code": "BAD",
            "name": "Bad",
            "symbol": "?",
            "asset_scale": 2,
            "asset_type": "gold",
            "caip19_id": "swift:0/iso4217:BAD"
        }))
        .await;
    resp.assert_status_bad_request();
}

#[tokio::test]
async fn update_currency_name_and_symbol() {
    let app = common::TestApp::new().await;
    let usd_id = app.create_usd().await;

    let resp = app
        .server
        .patch(&format!("/api/v1/currencies/{usd_id}"))
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({"name": "United States Dollar", "symbol": "US$"}))
        .await
        .json::<serde_json::Value>();

    assert_eq!(resp["data"]["name"], "United States Dollar");
    assert_eq!(resp["data"]["symbol"], "US$");
    // Immutable fields unchanged
    assert_eq!(resp["data"]["code"], "USD");
}

#[tokio::test]
async fn get_nonexistent_currency_returns_404() {
    let app = common::TestApp::new().await;

    let resp = app
        .server
        .get("/api/v1/currencies/nonexistent-id")
        .add_header(app.auth_name(), app.auth_value())
        .await;
    resp.assert_status_not_found();
}

#[tokio::test]
async fn crypto_currency_with_high_precision() {
    let app = common::TestApp::new().await;

    let resp = app
        .create_currency("ETH", "Ether", "Ξ", 18, "crypto")
        .await;
    assert_eq!(resp["data"]["asset_scale"], 18);
    assert_eq!(resp["data"]["asset_type"], "crypto");
}
