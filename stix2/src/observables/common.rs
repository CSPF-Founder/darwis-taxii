//! Common properties and utilities for STIX Cyber Observable Objects (SCOs).
//!
//! This module provides common functionality shared by all SCOs, including:
//! - Common optional properties (object_marking_refs, granular_markings, extensions)
//! - Deterministic ID generation based on ID contributing properties
//! - ID contributing properties trait for each SCO type

use crate::canonicalization::canonicalize;
use crate::core::error::Result;
use crate::core::id::Identifier;
use crate::markings::GranularMarking;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

/// Trait for STIX Cyber Observable Objects that have ID contributing properties.
///
/// ID contributing properties are used to generate deterministic UUIDv5 identifiers
/// for SCOs in STIX 2.1. When these properties match between two objects, they will
/// have the same identifier.
pub trait IdContributing {
    /// The properties that contribute to the deterministic ID.
    ///
    /// An empty slice means the SCO uses a random UUID (e.g., Process).
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str];

    /// Check if this SCO type uses deterministic IDs.
    fn uses_deterministic_id() -> bool {
        !Self::ID_CONTRIBUTING_PROPERTIES.is_empty()
    }
}

/// Common optional properties for all SCOs.
///
/// These properties are defined for all STIX Cyber Observable Objects
/// but are optional.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ScoCommonProperties {
    /// References to marking definitions that apply to this object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_marking_refs: Vec<Identifier>,

    /// Granular markings for specific properties.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granular_markings: Vec<GranularMarking>,

    /// Extensions for this object.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub extensions: IndexMap<String, Value>,
}

impl ScoCommonProperties {
    /// Create new empty SCO common properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an object marking reference.
    pub fn add_object_marking_ref(&mut self, marking_ref: Identifier) {
        self.object_marking_refs.push(marking_ref);
    }

    /// Add a granular marking.
    pub fn add_granular_marking(&mut self, marking: GranularMarking) {
        self.granular_markings.push(marking);
    }

    /// Add an extension.
    pub fn add_extension(&mut self, name: impl Into<String>, value: Value) {
        self.extensions.insert(name.into(), value);
    }
}

/// Generate a deterministic SCO ID based on ID contributing properties.
///
/// STIX 2.1 SCOs use UUIDv5 with the STIX namespace to generate deterministic IDs
/// based on the object's identifying properties. This ensures that two SCOs with
/// the same identifying properties will have the same ID.
///
/// # Arguments
///
/// * `object_type` - The STIX type (e.g., "ipv6-addr")
/// * `contributing_properties` - A JSON object containing only the ID contributing properties
///
/// # Example
///
/// ```rust,ignore
/// use serde_json::json;
/// let id = generate_sco_id("ipv6-addr", &json!({"value": "2001:db8::1"}))?;
/// ```
pub fn generate_sco_id(object_type: &str, contributing_properties: &Value) -> Result<Identifier> {
    // Canonicalize the contributing properties
    let canonical = canonicalize(contributing_properties)?;

    // Generate UUIDv5 with STIX namespace
    let uuid = Uuid::new_v5(&Identifier::stix_namespace(), canonical.as_bytes());

    Identifier::with_uuid(object_type, uuid)
}

/// Generate a deterministic SCO ID from a single string value.
///
/// This is a convenience function for SCOs that have a single ID contributing property
/// called "value" (e.g., IPv4Address, IPv6Address, DomainName, URL, MACAddress).
pub fn generate_sco_id_from_value(object_type: &str, value: &str) -> Result<Identifier> {
    generate_sco_id(object_type, &json!({"value": value}))
}

/// Generate a deterministic SCO ID from a single string property with a custom name.
///
/// This is useful for SCOs like Mutex (which uses "name") or AutonomousSystem (which uses "number").
pub fn generate_sco_id_from_property(
    object_type: &str,
    property_name: &str,
    property_value: &Value,
) -> Result<Identifier> {
    let mut props = serde_json::Map::new();
    props.insert(property_name.to_string(), property_value.clone());
    generate_sco_id(object_type, &Value::Object(props))
}

/// Macro to implement common SCO traits including deterministic ID generation.
#[macro_export]
macro_rules! impl_sco_with_id {
    ($type:ty, $type_str:literal, $id_prop:ident) => {
        impl $crate::core::traits::StixTyped for $type {
            const TYPE: &'static str = $type_str;
        }

        impl $crate::core::traits::Identifiable for $type {
            fn id(&self) -> &$crate::core::id::Identifier {
                &self.id
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sco_id_deterministic() {
        let id1 = generate_sco_id_from_value("ipv6-addr", "2001:db8::1").unwrap();
        let id2 = generate_sco_id_from_value("ipv6-addr", "2001:db8::1").unwrap();
        assert_eq!(id1.uuid(), id2.uuid());
    }

    #[test]
    fn test_generate_sco_id_different_values() {
        let id1 = generate_sco_id_from_value("ipv6-addr", "2001:db8::1").unwrap();
        let id2 = generate_sco_id_from_value("ipv6-addr", "2001:db8::2").unwrap();
        assert_ne!(id1.uuid(), id2.uuid());
    }

    #[test]
    fn test_generate_sco_id_different_types() {
        let id1 = generate_sco_id_from_value("ipv4-addr", "192.168.1.1").unwrap();
        let id2 = generate_sco_id_from_value("ipv6-addr", "192.168.1.1").unwrap();
        // UUIDs should be the same (based on value), but the type prefix differs
        assert_eq!(id1.uuid(), id2.uuid());
        assert_ne!(id1.object_type(), id2.object_type());
    }

    #[test]
    fn test_sco_common_properties() {
        let mut props = ScoCommonProperties::new();
        assert!(props.object_marking_refs.is_empty());
        assert!(props.granular_markings.is_empty());
        assert!(props.extensions.is_empty());

        let marking_ref = Identifier::new_marking_definition();
        props.add_object_marking_ref(marking_ref);
        assert_eq!(props.object_marking_refs.len(), 1);
    }
}
