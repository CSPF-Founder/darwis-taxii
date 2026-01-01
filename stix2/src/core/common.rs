//! Common properties shared by STIX objects.
//!
//! This module defines the common properties that appear across multiple
//! STIX object types, as well as helper types and macros.

use crate::core::external_reference::ExternalReference;
use crate::core::id::Identifier;
use crate::core::kill_chain_phase::KillChainPhase;
use crate::core::timestamp::Timestamp;
use crate::markings::GranularMarking;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Common properties for all STIX Domain Objects (SDOs).
///
/// These properties are inherited by all SDOs and provide metadata
/// about the object's creation, modification, and marking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommonProperties {
    /// The STIX specification version.
    #[serde(default = "default_spec_version")]
    pub spec_version: String,

    /// The ID of the identity that created this object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by_ref: Option<Identifier>,

    /// When the object was created.
    pub created: Timestamp,

    /// When the object was last modified.
    pub modified: Timestamp,

    /// Whether the object has been revoked.
    #[serde(default)]
    pub revoked: bool,

    /// Labels describing this object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,

    /// Confidence in the correctness of this object (0-100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,

    /// The language of the text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,

    /// External references to additional information.
    #[serde(default)]
    pub external_references: Vec<ExternalReference>,

    /// References to marking definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_marking_refs: Vec<Identifier>,

    /// Granular markings for specific properties.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granular_markings: Vec<GranularMarking>,

    /// Extensions for this object.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub extensions: IndexMap<String, Value>,

    /// Custom properties (x_ prefixed).
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub custom_properties: IndexMap<String, Value>,
}

fn default_spec_version() -> String {
    "2.1".to_string()
}

impl Default for CommonProperties {
    fn default() -> Self {
        let now = Timestamp::now();
        Self {
            spec_version: default_spec_version(),
            created_by_ref: None,
            created: now,
            modified: now,
            revoked: false,
            labels: Vec::new(),
            confidence: None,
            lang: None,
            external_references: Vec::new(),
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            extensions: IndexMap::new(),
            custom_properties: IndexMap::new(),
        }
    }
}

impl CommonProperties {
    /// Create new common properties with current timestamp.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the created_by_ref.
    pub fn with_created_by(mut self, identity_ref: Identifier) -> Self {
        self.created_by_ref = Some(identity_ref);
        self
    }

    /// Set the labels.
    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }

    /// Add a label.
    pub fn add_label(&mut self, label: impl Into<String>) {
        self.labels.push(label.into());
    }

    /// Set the confidence.
    pub fn with_confidence(mut self, confidence: u8) -> Self {
        self.confidence = Some(confidence.min(100));
        self
    }

    /// Add an external reference.
    pub fn add_external_reference(&mut self, reference: ExternalReference) {
        self.external_references.push(reference);
    }

    /// Add an object marking reference.
    pub fn add_object_marking_ref(&mut self, marking_ref: Identifier) {
        self.object_marking_refs.push(marking_ref);
    }

    /// Add an extension.
    pub fn add_extension(&mut self, name: impl Into<String>, value: Value) {
        self.extensions.insert(name.into(), value);
    }

    /// Get an extension.
    pub fn get_extension(&self, name: &str) -> Option<&Value> {
        self.extensions.get(name)
    }

    /// Set a custom property.
    pub fn set_custom_property(&mut self, key: impl Into<String>, value: Value) {
        let key = key.into();
        // Ensure custom properties have x_ prefix
        let key = if key.starts_with("x_") {
            key
        } else {
            format!("x_{key}")
        };
        self.custom_properties.insert(key, value);
    }

    /// Get a custom property.
    pub fn get_custom_property(&self, key: &str) -> Option<&Value> {
        self.custom_properties.get(key)
    }

    /// Update the modified timestamp to now.
    pub fn touch(&mut self) {
        self.modified = Timestamp::now();
    }
}

/// Optional properties that may appear on various objects.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OptionalCommonProperties {
    /// A human-readable name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// A human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Kill chain phases.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kill_chain_phases: Vec<KillChainPhase>,

    /// Aliases for this object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

impl OptionalCommonProperties {
    /// Create new optional properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a kill chain phase.
    pub fn add_kill_chain_phase(&mut self, phase: KillChainPhase) {
        self.kill_chain_phases.push(phase);
    }

    /// Add an alias.
    pub fn add_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into());
    }
}

/// A hash dictionary for storing multiple hash values.
pub type Hashes = IndexMap<String, String>;

/// Create a new Hashes map.
pub fn hashes() -> Hashes {
    IndexMap::new()
}

/// Validate that a hash value is correct for its algorithm.
pub fn validate_hash(algorithm: &str, value: &str) -> bool {
    let expected_len = match algorithm.to_uppercase().as_str() {
        "MD5" => 32,
        "SHA-1" | "SHA1" => 40,
        "SHA-256" | "SHA256" => 64,
        "SHA-512" | "SHA512" => 128,
        "SHA3-256" => 64,
        "SHA3-512" => 128,
        "SSDEEP" => return value.chars().all(|c| c.is_ascii() && !c.is_control()),
        "TLSH" => return value.len() >= 70 && value.chars().all(|c| c.is_ascii_hexdigit()),
        _ => return true, // Allow unknown algorithms
    };

    value.len() == expected_len && value.chars().all(|c| c.is_ascii_hexdigit())
}

/// Macro to implement common SDO traits for a struct.
#[macro_export]
macro_rules! impl_sdo_traits {
    ($type:ty, $type_str:literal) => {
        impl $crate::core::traits::StixTyped for $type {
            const TYPE: &'static str = $type_str;
        }

        impl $crate::core::traits::Identifiable for $type {
            fn id(&self) -> &$crate::core::id::Identifier {
                &self.id
            }
        }

        impl $crate::core::traits::StixDomainObject for $type {
            fn created(&self) -> &$crate::core::timestamp::Timestamp {
                &self.common.created
            }

            fn modified(&self) -> &$crate::core::timestamp::Timestamp {
                &self.common.modified
            }

            fn created_by_ref(&self) -> Option<&$crate::core::id::Identifier> {
                self.common.created_by_ref.as_ref()
            }

            fn revoked(&self) -> bool {
                self.common.revoked
            }

            fn confidence(&self) -> Option<u8> {
                self.common.confidence
            }

            fn lang(&self) -> Option<&str> {
                self.common.lang.as_deref()
            }

            fn object_marking_refs(&self) -> Option<&[$crate::core::id::Identifier]> {
                if self.common.object_marking_refs.is_empty() {
                    None
                } else {
                    Some(&self.common.object_marking_refs)
                }
            }

            fn granular_markings(&self) -> Option<&[$crate::markings::GranularMarking]> {
                if self.common.granular_markings.is_empty() {
                    None
                } else {
                    Some(&self.common.granular_markings)
                }
            }
        }
    };
}

/// Macro to implement common SCO traits for a struct.
#[macro_export]
macro_rules! impl_sco_traits {
    ($type:ty, $type_str:literal) => {
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

/// Macro to implement common builder methods for SDO/SRO builders.
///
/// This macro adds builder methods for common properties like `revoked`, `lang`,
/// `object_marking_ref`, `granular_marking`, and `external_reference`.
///
/// # Usage
///
/// ```ignore
/// impl_common_builder_methods!(IndicatorBuilder);
/// ```
#[macro_export]
macro_rules! impl_common_builder_methods {
    ($builder:ty) => {
        impl $builder {
            /// Set the revoked flag.
            pub fn revoked(mut self, revoked: bool) -> Self {
                self.common.revoked = revoked;
                self
            }

            /// Set the language.
            pub fn lang(mut self, lang: impl Into<String>) -> Self {
                self.common.lang = Some(lang.into());
                self
            }

            /// Add an object marking reference.
            pub fn object_marking_ref(mut self, marking_ref: $crate::core::id::Identifier) -> Self {
                self.common.object_marking_refs.push(marking_ref);
                self
            }

            /// Add a granular marking.
            pub fn granular_marking(mut self, marking: $crate::markings::GranularMarking) -> Self {
                self.common.granular_markings.push(marking);
                self
            }

            /// Add an external reference.
            pub fn external_reference(
                mut self,
                reference: $crate::core::ExternalReference,
            ) -> Self {
                self.common.external_references.push(reference);
                self
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_properties_default() {
        let props = CommonProperties::default();
        assert_eq!(props.spec_version, "2.1");
        assert!(!props.revoked);
    }

    #[test]
    fn test_custom_properties() {
        let mut props = CommonProperties::default();
        props.set_custom_property("test", serde_json::json!("value"));
        assert!(props.custom_properties.contains_key("x_test"));
    }

    #[test]
    fn test_validate_hash_md5() {
        assert!(validate_hash("MD5", "d41d8cd98f00b204e9800998ecf8427e"));
        assert!(!validate_hash("MD5", "invalid"));
    }

    #[test]
    fn test_validate_hash_sha256() {
        assert!(validate_hash(
            "SHA-256",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        ));
    }
}
