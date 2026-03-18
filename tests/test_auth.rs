mod common;

use axum::http::{HeaderName, HeaderValue};
use serde_json::{Value, json};

// ── Setup Flow ──────────────────────────────────────────────────

#[tokio::test]
async fn setup_status_returns_needs_setup_on_fresh_db() {
    // TestApp creates a test admin user, so we need a truly fresh DB
    // to test needs_setup=true. Instead, test that the endpoint exists
    // and returns needs_setup=false when users exist.
    let app = common::TestApp::new().await;

    let resp = app
        .server
        .get("/auth/setup/status")
        .await
        .json::<Value>();
    assert_eq!(resp["data"]["needs_setup"], false);
}

#[tokio::test]
async fn setup_creates_first_user_and_returns_tokens() {
    let app = common::TestApp::new_without_user().await;

    // Should need setup
    let status = app
        .server
        .get("/auth/setup/status")
        .await
        .json::<Value>();
    assert_eq!(status["data"]["needs_setup"], true);

    // Run setup
    let resp = app
        .server
        .post("/auth/setup")
        .json(&json!({
            "name": "Admin",
            "email": "admin@example.com",
            "password": "password123"
        }))
        .await;
    resp.assert_status_ok();
    let body = resp.json::<Value>();
    assert!(body["data"]["access_token"].is_string());
    assert!(body["data"]["refresh_token"].is_string());
    assert_eq!(body["data"]["token_type"], "Bearer");

    // The access token should work
    let access_token = body["data"]["access_token"].as_str().unwrap();
    let me_resp = app
        .server
        .get("/auth/me")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {access_token}")).unwrap(),
        )
        .await
        .json::<Value>();
    assert_eq!(me_resp["data"]["name"], "Admin");
    assert_eq!(me_resp["data"]["email"], "admin@example.com");
}

#[tokio::test]
async fn setup_rejects_second_call() {
    let app = common::TestApp::new_without_user().await;

    // First setup succeeds
    let resp = app
        .server
        .post("/auth/setup")
        .json(&json!({
            "name": "Admin",
            "email": "admin@example.com",
            "password": "password123"
        }))
        .await;
    resp.assert_status_ok();

    // Second setup fails
    let resp2 = app
        .server
        .post("/auth/setup")
        .json(&json!({
            "name": "Hacker",
            "email": "hacker@evil.com",
            "password": "password123"
        }))
        .await;
    resp2.assert_status_bad_request();
    let body = resp2.json::<Value>();
    assert!(body["message"].as_str().unwrap().contains("already been completed"));
}

#[tokio::test]
async fn setup_status_false_after_setup() {
    let app = common::TestApp::new_without_user().await;

    app.server
        .post("/auth/setup")
        .json(&json!({
            "name": "Admin",
            "email": "admin@example.com",
            "password": "password123"
        }))
        .await;

    let status = app
        .server
        .get("/auth/setup/status")
        .await
        .json::<Value>();
    assert_eq!(status["data"]["needs_setup"], false);
}

#[tokio::test]
async fn setup_validates_password_length() {
    let app = common::TestApp::new_without_user().await;

    let resp = app
        .server
        .post("/auth/setup")
        .json(&json!({
            "name": "Admin",
            "email": "admin@example.com",
            "password": "short"
        }))
        .await;
    resp.assert_status_bad_request();
    let body = resp.json::<Value>();
    assert!(body["message"].as_str().unwrap().contains("8 characters"));
}

// ── CLI API Key Resolution ───────────────────────────────────────

#[test]
fn cli_resolve_user_id_with_valid_api_key() {
    let mut conn = clawcounting::db::connection::setup_connection(":memory:")
        .expect("Failed to open in-memory database");
    clawcounting::db::migrations::run_migrations(&mut conn)
        .expect("Failed to run migrations");

    // Create a service account
    let req = clawcounting::models::user::CreateServiceAccountRequest {
        name: "Test Agent".to_string(),
        permissions: json!({}),
    };
    let (user, api_key) = clawcounting::services::user_service::create_service_account(&conn, req)
        .expect("Failed to create service account");

    // Resolve via api key
    let resolved_id = clawcounting::cli::resolve_cli_user_id(&conn, Some(&api_key))
        .expect("Failed to resolve user ID");
    assert_eq!(resolved_id, user.id);
}

#[test]
fn cli_resolve_user_id_without_api_key_fails() {
    let mut conn = clawcounting::db::connection::setup_connection(":memory:")
        .expect("Failed to open in-memory database");
    clawcounting::db::migrations::run_migrations(&mut conn)
        .expect("Failed to run migrations");

    let result = clawcounting::cli::resolve_cli_user_id(&conn, None);
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(err.contains("API key required"));
}

#[test]
fn cli_resolve_user_id_with_invalid_api_key_fails() {
    let mut conn = clawcounting::db::connection::setup_connection(":memory:")
        .expect("Failed to open in-memory database");
    clawcounting::db::migrations::run_migrations(&mut conn)
        .expect("Failed to run migrations");

    let result = clawcounting::cli::resolve_cli_user_id(&conn, Some("tsk_invalid_key"));
    assert!(result.is_err());
}

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

    // Should include the test admin user
    let resp = app.get("/api/v1/users").await;
    let users = resp["data"].as_array().unwrap();
    assert!(users.len() >= 1); // test admin
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
