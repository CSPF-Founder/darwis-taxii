//! Database error types.

use thiserror::Error;

/// Database errors.
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// SQLx error.
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// Entity not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid data.
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Results not ready for async polling.
    ///
    /// This error is raised when content blocks are not immediately
    /// available and async polling should be used.
    #[error("Results not ready")]
    ResultsNotReady,
}

impl DatabaseError {
    /// Create a not found error with context.
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create an invalid data error.
    pub fn invalid_data<S: Into<String>>(msg: S) -> Self {
        Self::InvalidData(msg.into())
    }

    /// Check if this is a unique constraint violation.
    pub fn is_unique_violation(&self) -> bool {
        match self {
            Self::Sqlx(e) => e
                .as_database_error()
                .map(|de| de.is_unique_violation())
                .unwrap_or(false),
            _ => false,
        }
    }

    /// Check if this is a not found error.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }
}

/// Result type for database operations.
pub type DatabaseResult<T> = Result<T, DatabaseError>;
