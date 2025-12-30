//! Validation Context
//!
//! Provides the `ValidationContext` which controls validation behavior.

use crate::registry::SpecVersion;
use std::cell::RefCell;

/// Configuration for STIX object validation.
///
/// # Example
///
/// ```rust,ignore
/// use stix2::validation::ValidationContext;
///
/// // Strict mode (default) - rejects custom properties
/// let strict = ValidationContext::strict();
///
/// // Allow custom properties
/// let custom = ValidationContext::new().allow_custom(true);
///
/// // Interoperability mode - relaxed UUID validation
/// let interop = ValidationContext::new().interoperability(true);
/// ```
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Whether to accept custom properties (x_* prefixed).
    ///
    /// When `false`, custom properties are rejected with a `CustomContentError`.
    /// When `true`, they are accepted and the `has_custom` flag is set on the object.
    pub allow_custom: bool,

    /// Whether to use relaxed UUID validation.
    ///
    /// When `false` (default), UUIDs must be valid UUIDv4 (STIX 2.0)
    /// or any RFC 4122 variant (STIX 2.1).
    ///
    /// When `true`, only the format is validated:
    /// `^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$`
    pub interoperability: bool,

    /// The STIX specification version.
    ///
    /// Affects validation rules:
    /// - STIX 2.0: Dictionary keys must be 3-256 chars
    /// - STIX 2.1: Dictionary keys must be 1-250 chars
    /// - STIX 2.1: Property names must start with alpha char
    pub spec_version: SpecVersion,
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self {
            // Default to true to avoid breaking existing code
            // Individual validation functions respect this flag
            allow_custom: true,
            interoperability: false,
            spec_version: SpecVersion::V21,
        }
    }
}

impl ValidationContext {
    /// Create a new validation context with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a strict validation context that rejects custom content.
    pub fn strict() -> Self {
        Self {
            allow_custom: false,
            interoperability: false,
            spec_version: SpecVersion::V21,
        }
    }

    /// Set whether to allow custom properties.
    pub fn allow_custom(mut self, allow: bool) -> Self {
        self.allow_custom = allow;
        self
    }

    /// Set whether to use interoperability mode.
    pub fn interoperability(mut self, interop: bool) -> Self {
        self.interoperability = interop;
        self
    }

    /// Set the STIX specification version.
    pub fn spec_version(mut self, version: SpecVersion) -> Self {
        self.spec_version = version;
        self
    }

    /// Create a context for STIX 2.0.
    pub fn stix20() -> Self {
        Self {
            allow_custom: true,
            interoperability: false,
            spec_version: SpecVersion::V20,
        }
    }

    /// Create a context for STIX 2.1.
    pub fn stix21() -> Self {
        Self {
            allow_custom: true,
            interoperability: false,
            spec_version: SpecVersion::V21,
        }
    }
}

// Thread-local storage for current validation context
thread_local! {
    static CONTEXT: RefCell<ValidationContext> = RefCell::new(ValidationContext::default());
}

/// Execute a function with a specific validation context.
///
/// The context is restored after the function completes.
///
/// # Example
///
/// ```rust,ignore
/// use stix2::validation::{ValidationContext, with_context};
///
/// let result = with_context(ValidationContext::strict(), || {
///     // Validation inside here uses strict mode
///     parse_stix_object(json)
/// });
/// ```
pub fn with_context<F, R>(ctx: ValidationContext, f: F) -> R
where
    F: FnOnce() -> R,
{
    CONTEXT.with(|c| {
        let old = c.replace(ctx);
        let result = f();
        c.replace(old);
        result
    })
}

/// Get the current validation context.
pub fn current_context() -> ValidationContext {
    CONTEXT.with(|c| c.borrow().clone())
}

/// Set the current validation context.
///
/// Returns the previous context.
pub fn set_context(ctx: ValidationContext) -> ValidationContext {
    CONTEXT.with(|c| c.replace(ctx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_context() {
        let ctx = ValidationContext::default();
        assert!(ctx.allow_custom);
        assert!(!ctx.interoperability);
        assert_eq!(ctx.spec_version, SpecVersion::V21);
    }

    #[test]
    fn test_strict_context() {
        let ctx = ValidationContext::strict();
        assert!(!ctx.allow_custom);
        assert!(!ctx.interoperability);
    }

    #[test]
    fn test_builder_pattern() {
        let ctx = ValidationContext::new()
            .allow_custom(false)
            .interoperability(true)
            .spec_version(SpecVersion::V20);

        assert!(!ctx.allow_custom);
        assert!(ctx.interoperability);
        assert_eq!(ctx.spec_version, SpecVersion::V20);
    }

    #[test]
    fn test_with_context() {
        let original = current_context();
        assert!(original.allow_custom);

        let result = with_context(ValidationContext::strict(), || {
            let inner = current_context();
            assert!(!inner.allow_custom);
            42
        });

        assert_eq!(result, 42);

        // Context restored
        let after = current_context();
        assert!(after.allow_custom);
    }
}
