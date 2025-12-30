//! TAXII 1.x errors.

use thiserror::Error;

use crate::constants::StatusType;

/// TAXII 1.x result type.
pub type Taxii1xResult<T> = Result<T, Taxii1xError>;

/// TAXII 1.x error.
#[derive(Debug, Error)]
pub enum Taxii1xError {
    /// Status message failure.
    #[error("TAXII {status_type}: {message}")]
    StatusMessage {
        message: String,
        in_response_to: Option<String>,
        status_type: StatusType,
        status_detail: Option<String>,
    },

    /// Invalid request.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Unsupported content type.
    #[error("Unsupported content type: {0}")]
    UnsupportedContentType(String),

    /// Missing header.
    #[error("Missing required header: {0}")]
    MissingHeader(&'static str),

    /// XML parsing error with context.
    ///
    /// Provides structured information about where parsing failed:
    /// - `message`: Description of the parsing error
    /// - `element`: The XML element being parsed when the error occurred (if known)
    /// - `position`: Byte position in the input where the error occurred (if known)
    #[error("XML parsing error: {message}{}", format_xml_context(.element, .position))]
    XmlParse {
        /// Description of the parsing error.
        message: String,
        /// The XML element being parsed when the error occurred.
        element: Option<String>,
        /// Byte position in the input where the error occurred.
        position: Option<usize>,
    },

    /// XML serialization error with context.
    ///
    /// Provides structured information about serialization failures:
    /// - `message`: Description of the serialization error
    /// - `element`: The element being serialized when the error occurred (if known)
    #[error("XML serialization error: {message}{}", .element.as_ref().map(|e| format!(" (element: {})", e)).unwrap_or_default())]
    XmlSerialize {
        /// Description of the serialization error.
        message: String,
        /// The element being serialized when the error occurred.
        element: Option<String>,
    },

    /// Unsupported TAXII version.
    #[error("Unsupported TAXII version: {0}")]
    UnsupportedVersion(String),

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] taxii_db::DatabaseError),
}

/// Format XML error context for display.
fn format_xml_context(element: &Option<String>, position: &Option<usize>) -> String {
    match (element, position) {
        (Some(elem), Some(pos)) => format!(" (element: {}, position: {})", elem, pos),
        (Some(elem), None) => format!(" (element: {})", elem),
        (None, Some(pos)) => format!(" (position: {})", pos),
        (None, None) => String::new(),
    }
}

impl Taxii1xError {
    /// Create a failure status message.
    pub fn failure(message: impl Into<String>, in_response_to: Option<String>) -> Self {
        Self::StatusMessage {
            message: message.into(),
            in_response_to,
            status_type: StatusType::Failure,
            status_detail: None,
        }
    }

    /// Create a failure with status detail.
    pub fn failure_with_detail(
        message: impl Into<String>,
        in_response_to: Option<String>,
        status_detail: impl Into<String>,
    ) -> Self {
        Self::StatusMessage {
            message: message.into(),
            in_response_to,
            status_type: StatusType::Failure,
            status_detail: Some(status_detail.into()),
        }
    }

    /// Create a status error with a specific status type.
    pub fn status(
        status_type: StatusType,
        message: impl Into<String>,
        in_response_to: Option<String>,
    ) -> Self {
        Self::StatusMessage {
            message: message.into(),
            in_response_to,
            status_type,
            status_detail: None,
        }
    }

    /// Create a status error with a specific status type and detail.
    pub fn status_with_detail(
        status_type: StatusType,
        message: impl Into<String>,
        in_response_to: Option<String>,
        status_detail: impl Into<String>,
    ) -> Self {
        Self::StatusMessage {
            message: message.into(),
            in_response_to,
            status_type,
            status_detail: Some(status_detail.into()),
        }
    }

    /// Create an XML parsing error from a quick-xml deserialization error.
    ///
    /// The error message includes position information when available
    /// from the underlying quick-xml error.
    pub fn xml_parse(error: quick_xml::DeError) -> Self {
        Self::XmlParse {
            message: error.to_string(),
            element: None,
            position: None,
        }
    }

    /// Create an XML parsing error with a simple message.
    pub fn xml_parse_msg(message: impl Into<String>) -> Self {
        Self::XmlParse {
            message: message.into(),
            element: None,
            position: None,
        }
    }

    /// Create an XML serialization error from a quick-xml error.
    pub fn xml_serialize(error: quick_xml::SeError) -> Self {
        Self::XmlSerialize {
            message: error.to_string(),
            element: None,
        }
    }

    /// Create an XML serialization error with a simple message.
    pub fn xml_serialize_msg(message: impl Into<String>) -> Self {
        Self::XmlSerialize {
            message: message.into(),
            element: None,
        }
    }
}
