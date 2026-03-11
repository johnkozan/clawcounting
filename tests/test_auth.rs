mod common;

use axum::http::{HeaderName, HeaderValue};
use serde_json::{Value, json};

// ── Authentication ───────────────────────────────────────────────

#[tokio::test]
async fn unauthenticated_request_returns_401() {
    let app = common::TestApp::new().await;

    let resp = app.server.get("/api/v1/currencies").await;
    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn invalid_token_returns_401() {
    let app = common::TestApp::new().await;

    let resp = app
        .server
        .get("/api/v1/currencies")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_static("Bearer invalid-token"),
        )
        .await;
    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn valid_api_key_authenticates() {
    let app = common::TestApp::new().await;

    let resp = app
        .server
        .get("/api/v1/currencies")
        .add_header(app.auth_name(), app.auth_value())
        .await;
    resp.assert_status_ok();
}

#[tokio::test]
async fn health_endpoint_requires_no_auth() {
    let app = common::TestApp::new().await;

    let resp = app.server.get("/health").await;
    resp.assert_status_ok();
    let body = resp.json::<Value>();
    assert_eq!(body["status"], "ok");
}

// ── User Management ──────────────────────────────────────────────

#[tokio::test]
async fn create_human_user() {
    let app = common::TestApp::new().await;

    let resp = app.post(
        "/api/v1/users",
        &json!({
            "name": "Alice",
            "email": "alice@example.com",
            "password": "password123"
        }),
    ).await;

    assert_eq!(resp["data"]["name"], "Alice");
    assert_eq!(resp["data"]["email"], "alice@example.com");
    assert!(resp["data"]["id"].is_string());
    // password_hash should not be in response
    assert!(resp["data"]["password_hash"].is_null());
}

#[tokio::test]
async fn create_service_account_returns_api_key() {
    let app = common::TestApp::new().await;

    let resp = app.post(
        "/api/v1/users/service-accounts",
        &json!({
            "name": "AI Agent",
            "permissions": {"journals:create": true}
        }),
    ).await;

    let user = &resp["data"]["user"];
    assert_eq!(user["name"], "AI Agent");
    assert!(user["email"].is_null());

    let api_key = resp["data"]["api_key"].as_str().unwrap();
    assert!(api_key.starts_with("tsk_"));

    // The new API key should work for authentication
    let auth_resp = app
        .server
        .get("/api/v1/currencies")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {api_key}")).unwrap(),
        )
        .await;
    auth_resp.assert_status_ok();
}

#[tokio::test]
async fn list_users() {
    let app = common::TestApp::new().await;

    // Should include the system user and the test admin
    let resp = app.get("/api/v1/users").await;
    let users = resp["data"].as_array().unwrap();
    assert!(users.len() >= 2); // system user + test admin
}

#[tokio::test]
async fn update_user() {
    let app = common::TestApp::new().await;

    let create_resp = app.post(
        "/api/v1/users",
        &json!({
            "name": "Bob",
            "email": "bob@example.com",
            "password": "password123"
        }),
    ).await;
    let user_id = create_resp["data"]["id"].as_str().unwrap();

    let update_resp = app.patch(
        &format!("/api/v1/users/{user_id}"),
        &json!({"name": "Robert", "is_active": false}),
    ).await;

    assert_eq!(update_resp["data"]["name"], "Robert");
    assert_eq!(update_resp["data"]["is_active"], false);
}

#[tokio::test]
async fn duplicate_email_rejected() {
    let app = common::TestApp::new().await;

    app.post(
        "/api/v1/users",
        &json!({
            "name": "Alice",
            "email": "alice@example.com",
            "password": "password123"
        }),
    ).await;

    let resp = app
        .server
        .post("/api/v1/users")
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({
            "name": "Another Alice",
            "email": "alice@example.com",
            "password": "password456"
        }))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<Value>();
    assert!(body["message"].as_str().unwrap().contains("already in use"));
}

#[tokio::test]
async fn short_password_rejected() {
    let app = common::TestApp::new().await;

    let resp = app
        .server
        .post("/api/v1/users")
        .add_header(app.auth_name(), app.auth_value())
        .json(&json!({
            "name": "Alice",
            "email": "alice@example.com",
            "password": "short"
        }))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<Value>();
    assert!(body["message"].as_str().unwrap().contains("8 characters"));
}

// ── JWT Login Flow ───────────────────────────────────────────────

#[tokio::test]
async fn login_and_use_jwt() {
    let app = common::TestApp::new().await;

    // Create a user
    app.post(
        "/api/v1/users",
        &json!({
            "name": "Alice",
            "email": "alice@example.com",
            "password": "password123"
        }),
    ).await;

    // Login
    let login_resp = app
        .server
        .post("/auth/login")
        .json(&json!({
            "email": "alice@example.com",
            "password": "password123"
        }))
        .await
        .json::<Value>();

    let access_token = login_resp["data"]["access_token"].as_str().unwrap();
    let refresh_token = login_resp["data"]["refresh_token"].as_str().unwrap();
    assert_eq!(login_resp["data"]["token_type"], "Bearer");
    assert!(login_resp["data"]["expires_in"].as_u64().unwrap() > 0);

    // Use access token to access protected endpoint
    let resp = app
        .server
        .get("/api/v1/currencies")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {access_token}")).unwrap(),
        )
        .await;
    resp.assert_status_ok();

    // Use access token with /auth/me
    let me_resp = app
        .server
        .get("/auth/me")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {access_token}")).unwrap(),
        )
        .await
        .json::<Value>();
    assert_eq!(me_resp["data"]["email"], "alice@example.com");

    // Refresh token
    let refresh_resp = app
        .server
        .post("/auth/refresh")
        .json(&json!({"refresh_token": refresh_token}))
        .await
        .json::<Value>();
    assert!(refresh_resp["data"]["access_token"].is_string());
}

#[tokio::test]
async fn login_with_wrong_password_fails() {
    let app = common::TestApp::new().await;

    app.post(
        "/api/v1/users",
        &json!({
            "name": "Alice",
            "email": "alice@example.com",
            "password": "password123"
        }),
    ).await;

    let resp = app
        .server
        .post("/auth/login")
        .json(&json!({
            "email": "alice@example.com",
            "password": "wrongpassword"
        }))
        .await;
    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn login_with_nonexistent_email_fails() {
    let app = common::TestApp::new().await;

    let resp = app
        .server
        .post("/auth/login")
        .json(&json!({
            "email": "nobody@example.com",
            "password": "password123"
        }))
        .await;
    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn deactivated_user_cannot_authenticate() {
    let app = common::TestApp::new().await;

    // Create service account
    let sa_resp = app.post(
        "/api/v1/users/service-accounts",
        &json!({"name": "Bot"}),
    ).await;
    let user_id = sa_resp["data"]["user"]["id"].as_str().unwrap();
    let api_key = sa_resp["data"]["api_key"].as_str().unwrap();

    // Deactivate
    app.patch(
        &format!("/api/v1/users/{user_id}"),
        &json!({"is_active": false}),
    ).await;

    // Try to use the API key
    let resp = app
        .server
        .get("/api/v1/currencies")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {api_key}")).unwrap(),
        )
        .await;
    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn refresh_token_cannot_be_used_as_access_token() {
    let app = common::TestApp::new().await;

    app.post(
        "/api/v1/users",
        &json!({
            "name": "Alice",
            "email": "alice@example.com",
            "password": "password123"
        }),
    ).await;

    let login_resp = app
        .server
        .post("/auth/login")
        .json(&json!({
            "email": "alice@example.com",
            "password": "password123"
        }))
        .await
        .json::<Value>();

    let refresh_token = login_resp["data"]["refresh_token"].as_str().unwrap();

    // Using refresh token as access token should fail
    let resp = app
        .server
        .get("/api/v1/currencies")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {refresh_token}")).unwrap(),
        )
        .await;
    resp.assert_status_unauthorized();
}
