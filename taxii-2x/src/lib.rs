//! TAXII 2.x protocol implementation.

pub mod error;
pub mod handlers;
pub mod http;
pub mod responses;
pub mod state;
pub mod validation;

pub use error::{Taxii2Error, Taxii2Result};
pub use handlers::*;
pub use http::*;
pub use responses::*;
pub use state::{Taxii2Config, Taxii2State, enforce_pagination_limit};
pub use validation::ValidatedBundle;

// Re-export stix2 types for consumers
pub use stix2::{Bundle, Identifier, StixObject};
