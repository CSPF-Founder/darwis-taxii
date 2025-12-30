//! Error types for the STIX2 library.
//!
//! This module defines all error types that can occur during STIX object
//! creation, parsing, validation, and serialization.

use std::fmt;
use thiserror::Error;

/// Result type alias using the library's Error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the STIX2 library.
#[derive(Error, Debug)]
pub enum Error {
    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid STIX identifier format.
    #[error("Invalid STIX identifier: {0}")]
    InvalidId(String),

    /// Invalid STIX type.
    #[error("Invalid STIX type: {0}")]
    InvalidType(String),

    /// Missing required property.
    #[error("Missing required property: {0}")]
    MissingProperty(String),

    /// Invalid property value.
    #[error("Invalid property value for '{property}': {message}")]
    InvalidPropertyValue {
        /// The property name.
        property: String,
        /// The error message.
        message: String,
    },

    /// Mutually exclusive properties specified.
    #[error("Properties {0:?} are mutually exclusive")]
    MutuallyExclusiveProperties(Vec<String>),

    /// At least one of the properties is required.
    #[error("At least one of {0:?} must be provided")]
    AtLeastOneRequired(Vec<String>),

    /// Property dependency not satisfied.
    #[error("Property '{dependent}' requires '{dependency}' to be present")]
    PropertyDependency {
        /// The dependent property.
        dependent: String,
        /// The required dependency.
        dependency: String,
    },

    /// Invalid timestamp format.
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    /// Invalid UUID format.
    #[error("Invalid UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),

    /// Invalid URL format.
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// Validation error.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Pattern parsing error.
    #[error("Pattern parsing error: {0}")]
    PatternParse(String),

    /// Pattern validation error.
    #[error("Pattern validation error: {0}")]
    PatternValidation(String),

    /// Invalid hash format.
    #[error("Invalid hash format for algorithm '{algorithm}': {message}")]
    InvalidHash {
        /// The hash algorithm.
        algorithm: String,
        /// The error message.
        message: String,
    },

    /// Invalid base64 encoding.
    #[error("Invalid base64 encoding: {0}")]
    InvalidBase64(#[from] base64::DecodeError),

    /// Invalid hex encoding.
    #[error("Invalid hex encoding: {0}")]
    InvalidHex(#[from] hex::FromHexError),

    /// Invalid IP address.
    #[error("Invalid IP address: {0}")]
    InvalidIpAddress(String),

    /// Invalid CIDR notation.
    #[error("Invalid CIDR notation: {0}")]
    InvalidCidr(#[from] ipnetwork::IpNetworkError),

    /// Regex error.
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// DataStore error.
    #[error("DataStore error: {0}")]
    DataStore(String),

    /// Lock acquisition error.
    #[error("Failed to acquire {lock_type} lock: {context}")]
    LockError {
        /// The type of lock (read or write).
        lock_type: &'static str,
        /// Context about where the lock failure occurred.
        context: &'static str,
    },

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Object not found.
    #[error("Object not found: {0}")]
    NotFound(String),

    /// Duplicate object.
    #[error("Duplicate object: {0}")]
    Duplicate(String),

    /// Duplicate type registration.
    #[error("Duplicate type registration: {0}")]
    DuplicateType(String),

    /// Version conflict.
    #[error("Version conflict: {0}")]
    VersionConflict(String),

    /// Immutable property modification attempt.
    #[error("Cannot modify immutable property: {0}")]
    ImmutableProperty(String),

    /// Invalid reference.
    #[error("Invalid reference to '{reference}': {message}")]
    InvalidReference {
        /// The reference identifier.
        reference: String,
        /// The error message.
        message: String,
    },

    /// Invalid marking.
    #[error("Invalid marking: {0}")]
    InvalidMarking(String),

    /// Invalid selector for granular marking.
    #[error("Invalid selector: {0}")]
    InvalidSelector(String),

    /// Custom content error (when allow_custom=false).
    #[error("Custom content error: {0}")]
    CustomContentError(String),

    /// Dictionary key error.
    #[error("Dictionary key '{key}' is invalid: {reason}")]
    DictionaryKeyError {
        /// The invalid key.
        key: String,
        /// The reason for the error.
        reason: String,
    },

    /// Custom error with a message.
    #[error("{0}")]
    Custom(String),

    /// Builder error - required field not set.
    #[error("Builder error: required field '{0}' not set")]
    Builder(String),
}

impl Error {
    /// Create a new custom error.
    pub fn custom<S: Into<String>>(msg: S) -> Self {
        Error::Custom(msg.into())
    }

    /// Create a missing property error.
    pub fn missing_property<S: Into<String>>(property: S) -> Self {
        Error::MissingProperty(property.into())
    }

    /// Create an invalid property value error.
    pub fn invalid_property_value<S: Into<String>>(property: S, message: S) -> Self {
        Error::InvalidPropertyValue {
            property: property.into(),
            message: message.into(),
        }
    }

    /// Create a validation error.
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Error::Validation(msg.into())
    }

    /// Create a builder error.
    pub fn builder<S: Into<String>>(field: S) -> Self {
        Error::Builder(field.into())
    }

    /// Create a datastore error.
    pub fn datastore<S: Into<String>>(msg: S) -> Self {
        Error::DataStore(msg.into())
    }

    /// Create a not found error.
    pub fn not_found<S: Into<String>>(id: S) -> Self {
        Error::NotFound(id.into())
    }

    /// Create an IO error from a string message.
    pub fn io<S: Into<String>>(msg: S) -> Self {
        Error::Custom(format!("IO error: {}", msg.into()))
    }

    /// Create a serialization error.
    pub fn serialization<S: Into<String>>(msg: S) -> Self {
        Error::Custom(format!("Serialization error: {}", msg.into()))
    }

    /// Create a read lock error.
    pub fn read_lock(context: &'static str) -> Self {
        Error::LockError {
            lock_type: "read",
            context,
        }
    }

    /// Create a write lock error.
    pub fn write_lock(context: &'static str) -> Self {
        Error::LockError {
            lock_type: "write",
            context,
        }
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Custom(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Custom(s.to_string())
    }
}

/// A trait for converting errors into validation errors.
pub trait IntoValidationError {
    /// Convert this error into a validation error.
    fn into_validation_error(self) -> Error;
}

impl<E: fmt::Display> IntoValidationError for E {
    fn into_validation_error(self) -> Error {
        Error::Validation(self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::MissingProperty("name".to_string());
        assert_eq!(err.to_string(), "Missing required property: name");
    }

    #[test]
    fn test_error_custom() {
        let err = Error::custom("Custom error message");
        assert_eq!(err.to_string(), "Custom error message");
    }

    #[test]
    fn test_error_from_string() {
        let err: Error = "Test error".into();
        assert_eq!(err.to_string(), "Test error");
    }
}
