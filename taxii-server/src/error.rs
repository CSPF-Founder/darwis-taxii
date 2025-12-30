//! Server errors.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use tracing::error;

/// Server result type.
pub type ServerResult<T> = Result<T, ServerError>;

/// Server error.
#[derive(Debug, Error)]
pub enum ServerError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] taxii_db::DatabaseError),

    /// Auth error.
    #[error("Auth error: {0}")]
    Auth(#[from] taxii_auth::AuthError),

    /// TAXII 1.x error.
    #[error("TAXII 1.x error: {0}")]
    Taxii1x(#[from] taxii_1x::Taxii1xError),

    /// TAXII 2.x error.
    #[error("TAXII 2.x error: {0}")]
    Taxii2x(#[from] taxii_2x::Taxii2Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        // Log full error details internally for debugging
        error!("Server error: {:?}", self);

        // Handle Taxii2x separately - it has its own IntoResponse with proper formatting
        if let Self::Taxii2x(e) = self {
            return e.into_response();
        }

        // Return user-friendly messages only - no internal details exposed
        let (status, message) = match &self {
            Self::Config(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Server configuration error",
            ),
            Self::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred"),
            Self::Auth(_) => (StatusCode::UNAUTHORIZED, "Authentication failed"),
            Self::Taxii1x(_) => (StatusCode::BAD_REQUEST, "Invalid TAXII 1.x request"),
            Self::Taxii2x(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            Self::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Server I/O error"),
        };

        (status, message).into_response()
    }
}
