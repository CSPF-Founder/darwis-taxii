//! Relationship SRO
//!
//! A Relationship is a link between two STIX Domain Objects.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::impl_sdo_traits;
use crate::validation::{Constrained, check_timestamp_order_strict};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Relationship STIX Relationship Object.
///
/// The Relationship object is used to link together two SDOs or SCOs in order
/// to describe how they are related to each other.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    /// The type property identifies the type of STIX Object.
    #[serde(rename = "type")]
    pub type_: String,

    /// The id property uniquely identifies this object.
    pub id: Identifier,

    /// Common properties shared by all SDOs.
    #[serde(flatten)]
    pub common: CommonProperties,

    /// The type of relationship being expressed.
    pub relationship_type: String,

    /// A description that provides more details about the relationship.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The id of the source object.
    pub source_ref: Identifier,

    /// The id of the target object.
    pub target_ref: Identifier,

    /// When this relationship was first seen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<Timestamp>,

    /// When this relationship was last seen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_time: Option<Timestamp>,

    /// Extensions for this object.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub extensions: IndexMap<String, Value>,
}

impl Relationship {
    /// The STIX type identifier for Relationship.
    pub const TYPE: &'static str = "relationship";

    /// Create a new RelationshipBuilder.
    pub fn builder() -> RelationshipBuilder {
        RelationshipBuilder::new()
    }

    /// Create a simple relationship between two objects.
    pub fn new(
        relationship_type: impl Into<String>,
        source_ref: Identifier,
        target_ref: Identifier,
    ) -> Result<Self> {
        Self::builder()
            .relationship_type(relationship_type)
            .source_ref(source_ref)
            .target_ref(target_ref)
            .build()
    }

    /// Create an "indicates" relationship (e.g., Indicator indicates Malware).
    pub fn indicates(source_ref: Identifier, target_ref: Identifier) -> Result<Self> {
        Self::new("indicates", source_ref, target_ref)
    }

    /// Create a "uses" relationship (e.g., Threat Actor uses Malware).
    pub fn uses(source_ref: Identifier, target_ref: Identifier) -> Result<Self> {
        Self::new("uses", source_ref, target_ref)
    }

    /// Create a "targets" relationship (e.g., Campaign targets Identity).
    pub fn targets(source_ref: Identifier, target_ref: Identifier) -> Result<Self> {
        Self::new("targets", source_ref, target_ref)
    }

    /// Create an "attributed-to" relationship.
    pub fn attributed_to(source_ref: Identifier, target_ref: Identifier) -> Result<Self> {
        Self::new("attributed-to", source_ref, target_ref)
    }

    /// Create a "mitigates" relationship.
    pub fn mitigates(source_ref: Identifier, target_ref: Identifier) -> Result<Self> {
        Self::new("mitigates", source_ref, target_ref)
    }

    /// Create a "related-to" relationship (generic).
    pub fn related_to(source_ref: Identifier, target_ref: Identifier) -> Result<Self> {
        Self::new("related-to", source_ref, target_ref)
    }
}

impl_sdo_traits!(Relationship, "relationship");

impl Constrained for Relationship {
    /// Validate Relationship constraints.
    ///
    /// - `stop_time` must be > `start_time` (strict inequality)
    fn validate_constraints(&self) -> Result<()> {
        check_timestamp_order_strict(
            self.start_time.as_ref(),
            self.stop_time.as_ref(),
            "start_time",
            "stop_time",
        )
    }
}

/// Invalid source/target reference types for Relationship.
/// Per STIX 2.1 spec, these types cannot be used as source or target.
const INVALID_RELATIONSHIP_REF_TYPES: &[&str] = &[
    "bundle",
    "language-content",
    "marking-definition",
    "relationship",
    "sighting",
];

/// Builder for creating Relationship objects.
#[derive(Debug, Default)]
pub struct RelationshipBuilder {
    relationship_type: Option<String>,
    description: Option<String>,
    source_ref: Option<Identifier>,
    target_ref: Option<Identifier>,
    start_time: Option<Timestamp>,
    stop_time: Option<Timestamp>,
    extensions: IndexMap<String, Value>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(RelationshipBuilder);

impl RelationshipBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the relationship type (required).
    pub fn relationship_type(mut self, relationship_type: impl Into<String>) -> Self {
        self.relationship_type = Some(relationship_type.into());
        self
    }

    /// Set the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the source reference (required).
    pub fn source_ref(mut self, source_ref: Identifier) -> Self {
        self.source_ref = Some(source_ref);
        self
    }

    /// Set the target reference (required).
    pub fn target_ref(mut self, target_ref: Identifier) -> Self {
        self.target_ref = Some(target_ref);
        self
    }

    /// Set the start time.
    pub fn start_time(mut self, start_time: Timestamp) -> Self {
        self.start_time = Some(start_time);
        self
    }

    /// Set the stop time.
    pub fn stop_time(mut self, stop_time: Timestamp) -> Self {
        self.stop_time = Some(stop_time);
        self
    }

    /// Set the created_by_ref.
    pub fn created_by_ref(mut self, identity_ref: Identifier) -> Self {
        self.common.created_by_ref = Some(identity_ref);
        self
    }

    /// Add a label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.common.labels.push(label.into());
        self
    }

    /// Set confidence level.
    pub fn confidence(mut self, confidence: u8) -> Self {
        self.common.confidence = Some(confidence.min(100));
        self
    }

    /// Add an extension.
    pub fn extension(mut self, name: impl Into<String>, value: Value) -> Self {
        self.extensions.insert(name.into(), value);
        self
    }

    /// Build the Relationship.
    pub fn build(self) -> Result<Relationship> {
        let relationship_type = self
            .relationship_type
            .ok_or_else(|| Error::missing_property("relationship_type"))?;
        let source_ref = self
            .source_ref
            .ok_or_else(|| Error::missing_property("source_ref"))?;
        let target_ref = self
            .target_ref
            .ok_or_else(|| Error::missing_property("target_ref"))?;

        // Validate source_ref type
        let source_type = source_ref.object_type();
        if INVALID_RELATIONSHIP_REF_TYPES.contains(&source_type) {
            return Err(Error::InvalidPropertyValue {
                property: "source_ref".to_string(),
                message: format!(
                    "'{}' is not a valid source type for a relationship",
                    source_type
                ),
            });
        }

        // Validate target_ref type
        let target_type = target_ref.object_type();
        if INVALID_RELATIONSHIP_REF_TYPES.contains(&target_type) {
            return Err(Error::InvalidPropertyValue {
                property: "target_ref".to_string(),
                message: format!(
                    "'{}' is not a valid target type for a relationship",
                    target_type
                ),
            });
        }

        let relationship = Relationship {
            type_: Relationship::TYPE.to_string(),
            id: Identifier::new(Relationship::TYPE)?,
            common: self.common,
            relationship_type,
            description: self.description,
            source_ref,
            target_ref,
            start_time: self.start_time,
            stop_time: self.stop_time,
            extensions: self.extensions,
        };

        // Validate constraints
        relationship.validate_constraints()?;

        Ok(relationship)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_relationship() {
        let source: Identifier = "indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f"
            .parse()
            .unwrap();
        let target: Identifier = "malware--31b940d4-6f7f-459a-80ea-9c1f17b5891b"
            .parse()
            .unwrap();

        let rel = Relationship::indicates(source.clone(), target.clone()).unwrap();

        assert_eq!(rel.relationship_type, "indicates");
        assert_eq!(rel.source_ref, source);
        assert_eq!(rel.target_ref, target);
    }

    #[test]
    fn test_relationship_builder() {
        let source: Identifier = "threat-actor--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f"
            .parse()
            .unwrap();
        let target: Identifier = "malware--31b940d4-6f7f-459a-80ea-9c1f17b5891b"
            .parse()
            .unwrap();

        let rel = Relationship::builder()
            .relationship_type("uses")
            .source_ref(source)
            .target_ref(target)
            .description("Threat Actor uses this malware")
            .confidence(85)
            .build()
            .unwrap();

        assert_eq!(rel.relationship_type, "uses");
        assert_eq!(rel.common.confidence, Some(85));
    }

    #[test]
    fn test_serialization() {
        let source: Identifier = "indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f"
            .parse()
            .unwrap();
        let target: Identifier = "malware--31b940d4-6f7f-459a-80ea-9c1f17b5891b"
            .parse()
            .unwrap();

        let rel = Relationship::indicates(source, target).unwrap();

        let json = serde_json::to_string(&rel).unwrap();
        let parsed: Relationship = serde_json::from_str(&json).unwrap();
        assert_eq!(rel.relationship_type, parsed.relationship_type);
    }
}
