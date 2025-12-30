//! Auth errors.

use thiserror::Error;

/// Auth result type.
pub type AuthResult<T> = Result<T, AuthError>;

/// Auth error.
#[derive(Debug, Error)]
pub enum AuthError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] taxii_db::DatabaseError),

    /// JSON error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// JWT error.
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    /// Password error.
    #[error("Password error: {0}")]
    Password(String),

    /// Invalid permission error.
    #[error("Invalid permission: {0}")]
    InvalidPermission(String),
}
