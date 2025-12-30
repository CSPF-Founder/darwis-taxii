//! Repository layer for TAXII database operations.
//!
//! This module provides trait-based abstractions for database access,
//! enabling mockability for testing and clean separation of concerns.
//!
//! # Architecture
//!
//! - **Traits**: [`Taxii1Repository`] and [`Taxii2Repository`] define the interface
//! - **Implementations**: [`DbTaxii1Repository`] and [`DbTaxii2Repository`] provide PostgreSQL implementations
//! - **Conversions**: `From` implementations for model-to-entity transformations

pub mod conversions;
pub mod taxii1;
pub mod taxii2;
pub mod traits;

// Conversions are used via From trait, no need to re-export
pub use taxii1::DbTaxii1Repository;
pub use taxii2::{DbTaxii2Repository, get_object_version};
pub use traits::{Taxii1Repository, Taxii2Repository};
