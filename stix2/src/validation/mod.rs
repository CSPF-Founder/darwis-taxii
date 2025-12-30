//! STIX Object Validation System
//!
//! This module provides comprehensive validation for STIX objects.
//!
//! ## Overview
//!
//! The validation system includes:
//! - Property validation with type coercion
//! - Inter-property constraint checking
//! - Custom property (x_*) handling via `allow_custom`
//! - Relaxed UUID validation via `interoperability` mode

pub mod constraints;
pub mod context;
#[macro_use]
pub mod macros;
pub mod properties;

pub use constraints::*;
pub use context::*;
pub use properties::*;

use crate::core::error::Result;

/// Trait for objects with constraint validation.
///
/// Types implementing this trait can validate their internal consistency,
/// such as temporal ordering, mutual exclusivity, and dependencies.
pub trait Constrained {
    /// Validate all constraints for this object.
    ///
    /// Called after all properties are set but before the object is returned.
    /// Returns an error if any constraint is violated.
    fn validate_constraints(&self) -> Result<()>;
}

/// Trait for objects that track custom content.
pub trait CustomTracking {
    /// Returns true if this object contains custom content.
    fn has_custom(&self) -> bool;
}
