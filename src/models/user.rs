use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// User role, corresponding to the Role enum in the Flutter app
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[repr(i32)]
pub enum Role {
    Basic = 0,
    Privileged = 1,
    Admin = 2,
}

impl Default for Role {
    fn default() -> Self {
        Self::Basic
    }
}

/// Database user model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub role: Role,
    pub api_key: Option<String>,
    pub password_hash: Option<String>,
    pub last_edit: DateTime<Utc>,
}

/// JSON representation of a user for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub id: i64,
    pub name: String,
    pub role: Role,
    pub last_edit: DateTime<Utc>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            role: user.role,
            last_edit: user.last_edit,
        }
    }
}

/// Authentication request from a client
#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub api_key: String,
}

/// Authentication response to a client
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub authenticated: bool,
    pub id: Option<i64>,
    pub name: Option<String>,
    pub role: Option<Role>,
    pub last_edit: Option<DateTime<Utc>>,
}

impl AuthResponse {
    pub fn success(user: User) -> Self {
        Self {
            authenticated: true,
            id: Some(user.id),
            name: Some(user.name),
            role: Some(user.role),
            last_edit: Some(user.last_edit),
        }
    }

    pub fn failure() -> Self {
        Self {
            authenticated: false,
            id: None,
            name: None,
            role: None,
            last_edit: None,
        }
    }
}
