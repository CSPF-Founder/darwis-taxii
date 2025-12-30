//! TAXII 2.x HTTP helpers.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tracing::error;

/// TAXII 2.x content type.
pub const TAXII2_CONTENT_TYPE: &str = "application/taxii+json;version=2.1";

/// STIX 2.1 content type.
pub const STIX21_CONTENT_TYPE: &str = "application/stix+json;version=2.1";

/// Valid accept mimetypes for TAXII 2.x.
pub const VALID_ACCEPT_MIMETYPES: &[&str] = &[
    "application/taxii+json",
    "application/taxii+json;version=2.1",
    "*/*",
];

/// Valid content types for POST requests.
pub const VALID_CONTENT_TYPES: &[&str] = &["application/taxii+json;version=2.1"];

/// TAXII 2.x JSON response.
pub struct Taxii2Response<T: Serialize> {
    pub data: T,
    pub status: StatusCode,
    pub extra_headers: Vec<(String, String)>,
}

impl<T: Serialize> Taxii2Response<T> {
    /// Create a new TAXII 2.x response.
    pub fn new(data: T) -> Self {
        Self {
            data,
            status: StatusCode::OK,
            extra_headers: Vec::new(),
        }
    }

    /// Create a response with custom status.
    pub fn with_status(data: T, status: StatusCode) -> Self {
        Self {
            data,
            status,
            extra_headers: Vec::new(),
        }
    }

    /// Add extra headers.
    pub fn with_headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.extra_headers = headers;
        self
    }
}

impl<T: Serialize> IntoResponse for Taxii2Response<T> {
    fn into_response(self) -> Response {
        // Properly handle serialization errors instead of silently failing
        let body = match serde_json::to_string(&self.data) {
            Ok(json) => json,
            Err(e) => {
                error!("Failed to serialize TAXII 2.x response: {}", e);
                // Return error response instead of empty body
                return crate::error::Taxii2Error::Internal(
                    "Response serialization failed".to_string(),
                )
                .into_response();
            }
        };

        let mut response = (
            self.status,
            [(axum::http::header::CONTENT_TYPE, TAXII2_CONTENT_TYPE)],
            body,
        )
            .into_response();

        // Add extra headers
        let headers = response.headers_mut();
        for (key, value) in &self.extra_headers {
            if let (Ok(name), Ok(val)) = (
                axum::http::header::HeaderName::try_from(key.as_str()),
                axum::http::header::HeaderValue::from_str(value),
            ) {
                headers.insert(name, val);
            }
        }

        response
    }
}

/// Empty TAXII 2.x response.
pub struct EmptyTaxii2Response {
    pub status: StatusCode,
}

impl EmptyTaxii2Response {
    pub fn new() -> Self {
        Self {
            status: StatusCode::OK,
        }
    }

    pub fn with_status(status: StatusCode) -> Self {
        Self { status }
    }
}

impl Default for EmptyTaxii2Response {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoResponse for EmptyTaxii2Response {
    fn into_response(self) -> Response {
        (
            self.status,
            [(axum::http::header::CONTENT_TYPE, TAXII2_CONTENT_TYPE)],
            "",
        )
            .into_response()
    }
}
