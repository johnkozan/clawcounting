use serde::{Deserialize, Serialize};

/// Internal user model (includes hashes — never return directly to clients).
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub api_key_hash: Option<String>,
    pub permissions: String,
    pub is_active: bool,
    pub created_at: String,
}

/// Public user response (sensitive fields stripped).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub permissions: serde_json::Value,
    pub is_active: bool,
    pub created_at: String,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        UserResponse {
            id: u.id,
            name: u.name,
            email: u.email,
            permissions: serde_json::from_str(&u.permissions).unwrap_or_default(),
            is_active: u.is_active,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    #[serde(default = "default_permissions")]
    pub permissions: serde_json::Value,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateServiceAccountRequest {
    pub name: String,
    #[serde(default = "default_permissions")]
    pub permissions: serde_json::Value,
}

/// Returned on service account creation — includes plaintext API key (shown once).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ServiceAccountCreatedResponse {
    pub user: UserResponse,
    pub api_key: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub permissions: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RefreshResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

fn default_permissions() -> serde_json::Value {
    serde_json::json!({})
}
