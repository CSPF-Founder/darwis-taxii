//! Common error types.

use thiserror::Error;

/// Common TAXII errors.
#[derive(Debug, Error)]
pub enum TaxiiError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
