//! Core traits for STIX objects.
//!
//! This module defines the fundamental traits that all STIX objects implement,
//! providing a common interface for working with different object types.

use crate::core::error::Result;
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::markings::GranularMarking;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

/// Core trait for all STIX objects.
///
/// This trait provides the common interface for all STIX objects,
/// including SDOs, SROs, SCOs, and marking definitions.
pub trait StixTyped: Debug + Clone + Serialize + DeserializeOwned {
    /// The STIX type string (e.g., "indicator", "malware").
    const TYPE: &'static str;

    /// Get the STIX type string.
    fn stix_type(&self) -> &'static str {
        Self::TYPE
    }
}

/// Trait for objects that have an identifier.
pub trait Identifiable: StixTyped {
    /// Get the object's identifier.
    fn id(&self) -> &Identifier;
}

/// Trait for STIX Domain Objects (SDOs) and Relationship Objects (SROs).
///
/// These objects have common properties like `created`, `modified`,
/// `created_by_ref`, etc.
pub trait StixDomainObject: Identifiable {
    /// Get the creation timestamp.
    fn created(&self) -> &Timestamp;

    /// Get the modification timestamp.
    fn modified(&self) -> &Timestamp;

    /// Get the ID of the identity that created this object.
    fn created_by_ref(&self) -> Option<&Identifier>;

    /// Check if this object has been revoked.
    fn revoked(&self) -> bool;

    /// Get the confidence level (0-100).
    fn confidence(&self) -> Option<u8>;

    /// Get the language of the text content.
    fn lang(&self) -> Option<&str>;

    /// Get the object marking references.
    fn object_marking_refs(&self) -> Option<&[Identifier]>;

    /// Get the granular markings.
    fn granular_markings(&self) -> Option<&[GranularMarking]>;
}

/// Trait for STIX Cyber Observable Objects (SCOs).
///
/// SCOs represent observed cyber data like files, IP addresses, etc.
pub trait StixCyberObservable: Identifiable {
    /// Get the spec version (e.g., "2.1").
    fn spec_version(&self) -> &str {
        "2.1"
    }

    /// Check if the object is defanged.
    fn defanged(&self) -> bool {
        false
    }

    /// Get the object marking references.
    fn object_marking_refs(&self) -> Option<&[Identifier]>;

    /// Get the granular markings.
    fn granular_markings(&self) -> Option<&[GranularMarking]>;

    /// Generate a deterministic ID from the object's identifying properties.
    ///
    /// STIX 2.1 SCOs use deterministic IDs based on their content.
    fn generate_id(&self) -> Result<Identifier>;
}

/// Trait for versioned objects.
///
/// STIX SDOs and SROs support versioning through the `modified` property
/// and `new_version` / `revoke` operations.
pub trait Versioned: StixDomainObject + Sized {
    /// Create a new version of this object with updated properties.
    fn new_version(&self) -> Result<Self>;

    /// Create a revoked version of this object.
    fn revoke(&self) -> Result<Self>;
}

/// Trait for objects that support external references.
pub trait HasExternalReferences {
    /// Get the external references.
    fn external_references(&self) -> Option<&[crate::core::ExternalReference]>;
}

/// Trait for objects that support labels.
pub trait HasLabels {
    /// Get the labels.
    fn labels(&self) -> Option<&[String]>;
}

/// Trait for objects with kill chain phases.
pub trait HasKillChainPhases {
    /// Get the kill chain phases.
    fn kill_chain_phases(&self) -> Option<&[crate::core::KillChainPhase]>;
}

/// Trait for objects that can be serialized to JSON.
pub trait ToJson: Serialize {
    /// Serialize to a JSON string.
    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    /// Serialize to a pretty-printed JSON string.
    fn to_json_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    /// Serialize to a serde_json::Value.
    fn to_value(&self) -> Result<serde_json::Value> {
        serde_json::to_value(self).map_err(Into::into)
    }
}

// Blanket implementation for all Serialize types
impl<T: Serialize> ToJson for T {}

/// Trait for objects that can be deserialized from JSON.
pub trait FromJson: DeserializeOwned {
    /// Deserialize from a JSON string.
    fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Deserialize from a serde_json::Value.
    fn from_value(value: serde_json::Value) -> Result<Self> {
        serde_json::from_value(value).map_err(Into::into)
    }
}

// Blanket implementation for all DeserializeOwned types
impl<T: DeserializeOwned> FromJson for T {}

/// Trait for validating STIX objects.
pub trait Validate {
    /// Validate the object according to STIX specification.
    fn validate(&self) -> Result<()>;
}

/// Trait for objects that can compute a canonical representation.
///
/// This is used for deterministic ID generation and comparison.
pub trait Canonicalize {
    /// Get the canonical JSON representation.
    fn canonicalize(&self) -> Result<String>;
}

/// Trait for comparing STIX objects for semantic equivalence.
pub trait Equivalent {
    /// Check if this object is semantically equivalent to another.
    fn is_equivalent(&self, other: &Self) -> bool;

    /// Compute a similarity score (0.0 to 1.0) with another object.
    fn similarity(&self, other: &Self) -> f64;
}

/// Builder pattern trait for STIX objects.
pub trait Builder: Sized {
    /// The type of object this builder creates.
    type Output;

    /// Build the object, returning an error if required fields are missing.
    fn build(self) -> Result<Self::Output>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_json_trait() {
        #[derive(Debug, Clone, Serialize, serde::Deserialize)]
        struct TestObj {
            value: i32,
        }

        let obj = TestObj { value: 42 };
        let json = obj.to_json().unwrap();
        assert!(json.contains("42"));
    }
}
