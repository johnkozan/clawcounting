use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::user::{
    CreateServiceAccountRequest, CreateUserRequest, UpdateUserRequest, User, UserResponse,
};

// ── Password hashing (argon2) ──────────────────────────────────────

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))?;
    Ok(hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash format: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

// ── API key generation (SHA-256 hash for lookups) ──────────────────

pub fn generate_api_key() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    format!("tsk_{}", URL_SAFE_NO_PAD.encode(bytes))
}

pub fn hash_api_key(key: &str) -> String {
    let hash = Sha256::digest(key.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

// ── JWT helpers ────────────────────────────────────────────────────

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

pub const ACCESS_TOKEN_EXPIRY_SECS: u64 = 3600; // 1 hour
pub const REFRESH_TOKEN_EXPIRY_SECS: u64 = 604800; // 7 days

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub token_type: String,
}

pub fn create_access_token(user_id: &str, secret: &str) -> Result<String, AppError> {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        exp: now + ACCESS_TOKEN_EXPIRY_SECS as usize,
        iat: now,
        token_type: "access".to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("JWT creation failed: {e}")))
}

pub fn create_refresh_token(user_id: &str, secret: &str) -> Result<String, AppError> {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        exp: now + REFRESH_TOKEN_EXPIRY_SECS as usize,
        iat: now,
        token_type: "refresh".to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("JWT creation failed: {e}")))
}

pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;
    Ok(data.claims)
}

// ── Permission checking ────────────────────────────────────────────

pub fn require_permission(user: &User, permission: &str) -> Result<(), AppError> {
    let perms: serde_json::Value =
        serde_json::from_str(&user.permissions).unwrap_or(serde_json::json!({}));

    if perms.get("admin") == Some(&serde_json::json!(true)) {
        return Ok(());
    }
    if perms.get(permission) == Some(&serde_json::json!(true)) {
        return Ok(());
    }

    Err(AppError::Forbidden)
}

// ── CRUD ───────────────────────────────────────────────────────────

fn row_to_user(row: &rusqlite::Row) -> rusqlite::Result<User> {
    Ok(User {
        id: row.get("id")?,
        name: row.get("name")?,
        email: row.get("email")?,
        password_hash: row.get("password_hash")?,
        api_key_hash: row.get("api_key_hash")?,
        permissions: row.get("permissions")?,
        is_active: row.get("is_active")?,
        created_at: row.get("created_at")?,
    })
}

pub fn create_user(conn: &Connection, req: CreateUserRequest) -> Result<UserResponse, AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::ValidationError {
            field: "name".into(),
            message: "Name cannot be empty".into(),
            suggestion: "Provide a non-empty name".into(),
        });
    }
    if req.email.trim().is_empty() {
        return Err(AppError::ValidationError {
            field: "email".into(),
            message: "Email cannot be empty".into(),
            suggestion: "Provide a valid email address".into(),
        });
    }
    if req.password.len() < 8 {
        return Err(AppError::ValidationError {
            field: "password".into(),
            message: "Password must be at least 8 characters".into(),
            suggestion: "Choose a longer password".into(),
        });
    }

    // Check email uniqueness
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM users WHERE email = ?1",
            params![req.email],
            |row| row.get(0),
        )
        .optional()?;
    if existing.is_some() {
        return Err(AppError::ValidationError {
            field: "email".into(),
            message: "Email already in use".into(),
            suggestion: "Use a different email address".into(),
        });
    }

    let id = Uuid::now_v7().to_string();
    let pw_hash = hash_password(&req.password)?;
    let perms = req.permissions.to_string();

    conn.execute(
        "INSERT INTO users (id, name, email, password_hash, permissions)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, req.name, req.email, pw_hash, perms],
    )?;

    let user = get_user_internal(conn, &id)?;
    Ok(user.into())
}

pub fn create_service_account(
    conn: &Connection,
    req: CreateServiceAccountRequest,
) -> Result<(UserResponse, String), AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::ValidationError {
            field: "name".into(),
            message: "Name cannot be empty".into(),
            suggestion: "Provide a non-empty name".into(),
        });
    }

    let id = Uuid::now_v7().to_string();
    let api_key = generate_api_key();
    let key_hash = hash_api_key(&api_key);
    let perms = req.permissions.to_string();

    conn.execute(
        "INSERT INTO users (id, name, api_key_hash, permissions)
         VALUES (?1, ?2, ?3, ?4)",
        params![id, req.name, key_hash, perms],
    )?;

    let user = get_user_internal(conn, &id)?;
    Ok((user.into(), api_key))
}

pub fn get_user_internal(conn: &Connection, id: &str) -> Result<User, AppError> {
    conn.query_row("SELECT * FROM users WHERE id = ?1", params![id], row_to_user)
        .map_err(|_| AppError::NotFound {
            resource: "User".into(),
            id: id.to_string(),
        })
}

pub fn get_user(conn: &Connection, id: &str) -> Result<UserResponse, AppError> {
    Ok(get_user_internal(conn, id)?.into())
}

pub fn list_users(
    conn: &Connection,
    limit: u32,
    cursor: Option<&str>,
) -> Result<(Vec<UserResponse>, bool, Option<String>), AppError> {
    let fetch_limit = limit + 1;

    let (rows, _has_more, _next_cursor) = if let Some(cursor) = cursor {
        let mut stmt = conn.prepare(
            "SELECT * FROM users WHERE id > ?1 ORDER BY id LIMIT ?2",
        )?;
        let users: Vec<User> = stmt
            .query_map(params![cursor, fetch_limit], row_to_user)?
            .collect::<Result<_, _>>()?;
        let has_more = users.len() as u32 > limit;
        let next = if has_more {
            users.get(limit as usize - 1).map(|u| u.id.clone())
        } else {
            None
        };
        (users, has_more, next)
    } else {
        let mut stmt =
            conn.prepare("SELECT * FROM users ORDER BY id LIMIT ?1")?;
        let users: Vec<User> = stmt
            .query_map(params![fetch_limit], row_to_user)?
            .collect::<Result<_, _>>()?;
        let has_more = users.len() as u32 > limit;
        let next = if has_more {
            users.get(limit as usize - 1).map(|u| u.id.clone())
        } else {
            None
        };
        (users, has_more, next)
    };

    let has_more = rows.len() as u32 > limit;
    let mut data: Vec<UserResponse> = rows.into_iter().map(|u| u.into()).collect();
    if has_more {
        data.truncate(limit as usize);
    }
    let next_cursor = if has_more {
        data.last().map(|u| u.id.clone())
    } else {
        None
    };
    Ok((data, has_more, next_cursor))
}

pub fn update_user(
    conn: &Connection,
    id: &str,
    req: UpdateUserRequest,
) -> Result<UserResponse, AppError> {
    // Ensure user exists
    let _existing = get_user_internal(conn, id)?;

    if let Some(ref name) = req.name {
        conn.execute("UPDATE users SET name = ?1 WHERE id = ?2", params![name, id])?;
    }
    if let Some(ref permissions) = req.permissions {
        let perms = permissions.to_string();
        conn.execute(
            "UPDATE users SET permissions = ?1 WHERE id = ?2",
            params![perms, id],
        )?;
    }
    if let Some(is_active) = req.is_active {
        conn.execute(
            "UPDATE users SET is_active = ?1 WHERE id = ?2",
            params![is_active, id],
        )?;
    }

    Ok(get_user_internal(conn, id)?.into())
}

// ── Setup (first-run) ─────────────────────────────────────────────

pub fn has_any_users(conn: &Connection) -> Result<bool, AppError> {
    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    Ok(count > 0)
}

/// Check if any human (password-bearing) users exist.
/// Service accounts (API-key-only) don't count — they can't log in to the web UI.
pub fn has_any_web_users(conn: &Connection) -> Result<bool, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE password_hash IS NOT NULL",
        [],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Resolve a raw API key to a user ID. Used by CLI commands that need auth.
pub fn get_user_id_by_api_key(conn: &Connection, api_key: &str) -> Result<String, AppError> {
    let key_hash = hash_api_key(api_key);
    let user = authenticate_by_api_key_hash(conn, &key_hash)?;
    Ok(user.id)
}

// ── Authentication ────────────────────────────────────────────────

pub fn authenticate_by_password(
    conn: &Connection,
    email: &str,
    password: &str,
) -> Result<User, AppError> {
    let user: User = conn
        .query_row(
            "SELECT * FROM users WHERE email = ?1",
            params![email],
            row_to_user,
        )
        .optional()?
        .ok_or(AppError::Unauthorized)?;

    if !user.is_active {
        return Err(AppError::Unauthorized);
    }

    let pw_hash = user.password_hash.as_deref().ok_or(AppError::Unauthorized)?;
    if !verify_password(password, pw_hash)? {
        return Err(AppError::Unauthorized);
    }

    Ok(user)
}

pub fn authenticate_by_api_key_hash(conn: &Connection, key_hash: &str) -> Result<User, AppError> {
    let user: User = conn
        .query_row(
            "SELECT * FROM users WHERE api_key_hash = ?1",
            params![key_hash],
            row_to_user,
        )
        .optional()?
        .ok_or(AppError::Unauthorized)?;

    if !user.is_active {
        return Err(AppError::Unauthorized);
    }

    Ok(user)
}
