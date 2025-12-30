//! TAXII 2.x errors.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use tracing::{debug, error, warn};

/// TAXII 2.x result type.
pub type Taxii2Result<T> = Result<T, Taxii2Error>;

/// TAXII 2.x error response body.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    pub http_status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// TAXII 2.x error.
#[derive(Debug, Error)]
pub enum Taxii2Error {
    /// Validation error.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Unauthorized.
    #[error("Unauthorized")]
    Unauthorized,

    /// Forbidden.
    #[error("Forbidden")]
    Forbidden,

    /// Bad request.
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Method not allowed.
    #[error("Method not allowed")]
    MethodNotAllowed,

    /// Not acceptable.
    #[error("Not acceptable")]
    NotAcceptable,

    /// Unsupported media type.
    #[error("Unsupported media type")]
    UnsupportedMediaType,

    /// Request entity too large.
    #[error("Request entity too large")]
    RequestEntityTooLarge,

    /// Internal server error.
    #[error("Internal server error: {0}")]
    Internal(String),

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] taxii_db::DatabaseError),

    /// JSON error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// STIX2 library error.
    #[error("STIX error: {0}")]
    Stix2(#[from] stix2::Error),
}

impl Taxii2Error {
    /// Get HTTP status code.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Self::NotAcceptable => StatusCode::NOT_ACCEPTABLE,
            Self::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::RequestEntityTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Json(_) => StatusCode::BAD_REQUEST,
            Self::Stix2(_) => StatusCode::BAD_REQUEST,
        }
    }

    /// Convert to error response.
    pub fn to_error_response(&self) -> ErrorResponse {
        ErrorResponse {
            title: self.title(),
            description: self.user_description(),
            error_id: None,
            error_code: None,
            http_status: self.status_code().as_u16(),
            external_details: None,
            details: None,
        }
    }

    /// Get user-safe description (no internal details exposed).
    fn user_description(&self) -> Option<String> {
        match self {
            // Safe to expose - these are client-facing messages
            Self::Validation(msg) => Some(msg.clone()),
            Self::NotFound(msg) => Some(msg.clone()),
            Self::BadRequest(msg) => Some(msg.clone()),

            // Generic messages for internal/sensitive errors
            Self::Database(_) => Some("A database error occurred".to_string()),
            Self::Internal(_) => Some("An internal error occurred".to_string()),
            Self::Json(_) => Some("Invalid JSON format".to_string()),
            Self::Stix2(_) => Some("Invalid STIX object format".to_string()),

            // No description needed for simple status errors
            Self::Unauthorized
            | Self::Forbidden
            | Self::MethodNotAllowed
            | Self::NotAcceptable
            | Self::UnsupportedMediaType
            | Self::RequestEntityTooLarge => None,
        }
    }

    fn title(&self) -> String {
        match self {
            Self::Validation(_) => "Validation Error",
            Self::NotFound(_) => "Not Found",
            Self::Unauthorized => "Unauthorized",
            Self::Forbidden => "Forbidden",
            Self::BadRequest(_) => "Bad Request",
            Self::MethodNotAllowed => "Method Not Allowed",
            Self::NotAcceptable => "Not Acceptable",
            Self::UnsupportedMediaType => "Unsupported Media Type",
            Self::RequestEntityTooLarge => "Payload Too Large",
            Self::Internal(_) => "Internal Server Error",
            Self::Database(_) => "Internal Server Error",
            Self::Json(_) => "Bad Request",
            Self::Stix2(_) => "STIX Validation Error",
        }
        .to_string()
    }
}

impl IntoResponse for Taxii2Error {
    fn into_response(self) -> Response {
        // Log errors with appropriate severity levels
        match &self {
            Self::Database(e) => error!("Database error: {:?}", e),
            Self::Internal(msg) => error!("Internal error: {}", msg),
            Self::Json(e) => warn!("JSON parsing error: {}", e),
            Self::Stix2(e) => warn!("STIX2 validation error: {}", e),
            _ => debug!("Client error: {:?}", self),
        }

        let status = self.status_code();

        // Properly handle serialization errors instead of silently failing
        let body = match serde_json::to_string(&self.to_error_response()) {
            Ok(json) => json,
            Err(e) => {
                error!("Failed to serialize error response: {}", e);
                // Return a minimal valid JSON error response
                format!(
                    r#"{{"title":"Internal Server Error","http_status":{}}}"#,
                    StatusCode::INTERNAL_SERVER_ERROR.as_u16()
                )
            }
        };

        (
            status,
            [(
                axum::http::header::CONTENT_TYPE,
                crate::http::TAXII2_CONTENT_TYPE,
            )],
            body,
        )
            .into_response()
    }
}
