//! STIX Data Markings
//!
//! This module provides data marking definitions for applying handling
//! and sharing guidance to STIX objects.
//!
//! ## Marking Operations
//!
//! Use the `operations` submodule for manipulating markings:
//!
//! ```rust,ignore
//! use stix2::markings::operations::*;
//!
//! // Add object-level marking
//! let new_refs = add_object_marking(&obj.object_marking_refs, marking_id);
//!
//! // Add granular marking
//! let new_gm = add_granular_marking(&obj.granular_markings, marking_id, vec!["description".into()]);
//! ```

pub mod operations;

use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::validation::Constrained;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

// Standard TLP marking definition UUIDs (compile-time constants)
const TLP_CLEAR_UUID: Uuid = uuid::uuid!("94868c89-83c2-464b-929b-a1a8aa3c8487");
const TLP_WHITE_UUID: Uuid = uuid::uuid!("613f2e26-407d-48c7-9eca-b8e91df99dc9");
const TLP_GREEN_UUID: Uuid = uuid::uuid!("34098fce-860f-48ae-8e50-ebd3cc5e41da");
const TLP_AMBER_UUID: Uuid = uuid::uuid!("f88d31f6-486f-44da-b317-01333bde0b82");
const TLP_AMBER_STRICT_UUID: Uuid = uuid::uuid!("826578e1-40a3-4b26-bf02-f8e3c5d7f8a8");
const TLP_RED_UUID: Uuid = uuid::uuid!("5e57c739-391a-4eb3-b6be-7d15ca92d5ed");

/// Traffic Light Protocol (TLP) marking levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TlpLevel {
    /// TLP:CLEAR (previously TLP:WHITE) - Information may be distributed without restriction.
    Clear,
    /// TLP:WHITE - Legacy, use CLEAR.
    White,
    /// TLP:GREEN - Information may be shared within the community.
    Green,
    /// TLP:AMBER - Information may be shared on a need-to-know basis.
    Amber,
    /// TLP:AMBER+STRICT - More restrictive than AMBER.
    #[serde(rename = "amber+strict")]
    AmberStrict,
    /// TLP:RED - Information is restricted to participants only.
    Red,
}

impl TlpLevel {
    /// Get the TLP level as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            TlpLevel::Clear => "clear",
            TlpLevel::White => "white",
            TlpLevel::Green => "green",
            TlpLevel::Amber => "amber",
            TlpLevel::AmberStrict => "amber+strict",
            TlpLevel::Red => "red",
        }
    }

    /// Get the standard marking definition ID for this TLP level.
    pub fn marking_definition_id(&self) -> Identifier {
        let uuid = match self {
            TlpLevel::Clear => TLP_CLEAR_UUID,
            TlpLevel::White => TLP_WHITE_UUID,
            TlpLevel::Green => TLP_GREEN_UUID,
            TlpLevel::Amber => TLP_AMBER_UUID,
            TlpLevel::AmberStrict => TLP_AMBER_STRICT_UUID,
            TlpLevel::Red => TLP_RED_UUID,
        };
        Identifier::marking_definition(uuid)
    }
}

/// TLP Marking definition type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TlpMarking {
    pub tlp: TlpLevel,
}

impl TlpMarking {
    /// Create a new TLP marking.
    pub fn new(level: TlpLevel) -> Self {
        Self { tlp: level }
    }

    /// Create TLP:CLEAR marking.
    pub fn clear() -> Self {
        Self::new(TlpLevel::Clear)
    }

    /// Create TLP:GREEN marking.
    pub fn green() -> Self {
        Self::new(TlpLevel::Green)
    }

    /// Create TLP:AMBER marking.
    pub fn amber() -> Self {
        Self::new(TlpLevel::Amber)
    }

    /// Create TLP:RED marking.
    pub fn red() -> Self {
        Self::new(TlpLevel::Red)
    }
}

/// Statement marking definition type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatementMarking {
    pub statement: String,
}

impl StatementMarking {
    /// Create a new statement marking.
    pub fn new(statement: impl Into<String>) -> Self {
        Self {
            statement: statement.into(),
        }
    }
}

/// The definition type within a marking definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "definition_type", content = "definition")]
pub enum MarkingType {
    #[serde(rename = "tlp")]
    Tlp(TlpMarking),
    #[serde(rename = "statement")]
    Statement(StatementMarking),
}

/// External Reference for linking to external sources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalReference {
    /// The source name.
    pub source_name: String,
    /// An optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// An optional URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Optional hashes.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub hashes: IndexMap<String, String>,
    /// An optional external ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
}

/// Marking Definition STIX Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkingDefinition {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(default = "default_spec_version")]
    pub spec_version: String,
    pub created: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by_ref: Option<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// External references for the marking definition.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_references: Vec<ExternalReference>,
    /// Object marking references for the marking definition.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_marking_refs: Vec<Identifier>,
    /// Granular markings for the marking definition.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granular_markings: Vec<GranularMarking>,
    /// Extensions for the marking definition.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub extensions: IndexMap<String, Value>,
    #[serde(flatten)]
    pub marking_type: MarkingType,
}

fn default_spec_version() -> String {
    "2.1".to_string()
}

impl MarkingDefinition {
    pub const TYPE: &'static str = "marking-definition";

    /// Create a TLP marking definition.
    pub fn tlp(level: TlpLevel) -> Self {
        Self {
            type_: Self::TYPE.to_string(),
            id: level.marking_definition_id(),
            spec_version: default_spec_version(),
            created: Timestamp::now(),
            created_by_ref: None,
            name: Some(format!("TLP:{}", level.as_str().to_uppercase())),
            external_references: Vec::new(),
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            extensions: IndexMap::new(),
            marking_type: MarkingType::Tlp(TlpMarking::new(level)),
        }
    }

    /// Create a statement marking definition.
    pub fn statement(statement: impl Into<String>) -> Result<Self> {
        Ok(Self {
            type_: Self::TYPE.to_string(),
            id: Identifier::new(Self::TYPE)?,
            spec_version: default_spec_version(),
            created: Timestamp::now(),
            created_by_ref: None,
            name: None,
            external_references: Vec::new(),
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            extensions: IndexMap::new(),
            marking_type: MarkingType::Statement(StatementMarking::new(statement)),
        })
    }
}

/// Granular marking for applying markings to specific properties.
///
/// Note: `lang` and `marking_ref` are mutually exclusive - exactly one must be present.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GranularMarking {
    /// The language of the text marked (mutually exclusive with marking_ref).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// The marking definition to apply (mutually exclusive with lang).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marking_ref: Option<Identifier>,
    /// The properties to which the marking applies.
    pub selectors: Vec<String>,
}

impl GranularMarking {
    /// Create a new granular marking with a marking reference.
    pub fn new(marking_ref: Identifier, selectors: Vec<String>) -> Self {
        Self {
            lang: None,
            marking_ref: Some(marking_ref),
            selectors,
        }
    }

    /// Create a granular marking with a language specification.
    pub fn with_lang(selectors: Vec<String>, lang: impl Into<String>) -> Self {
        Self {
            lang: Some(lang.into()),
            marking_ref: None,
            selectors,
        }
    }

    /// Validate this granular marking.
    pub fn validate(&self) -> Result<()> {
        self.validate_constraints()
    }
}

impl Constrained for GranularMarking {
    /// Validate GranularMarking constraints.
    ///
    /// - Exactly one of `lang` or `marking_ref` must be present (mutually exclusive)
    fn validate_constraints(&self) -> Result<()> {
        match (&self.lang, &self.marking_ref) {
            (Some(_), Some(_)) => {
                // Both present - error
                Err(Error::MutuallyExclusiveProperties(vec![
                    "lang".to_string(),
                    "marking_ref".to_string(),
                ]))
            }
            (None, None) => {
                // Neither present - error
                Err(Error::AtLeastOneRequired(vec![
                    "lang".to_string(),
                    "marking_ref".to_string(),
                ]))
            }
            _ => Ok(()),
        }
    }
}

/// Pre-defined TLP marking definitions.
pub mod tlp {
    use super::{MarkingDefinition, TlpLevel};

    /// Get the TLP:CLEAR marking definition.
    pub fn clear() -> MarkingDefinition {
        MarkingDefinition::tlp(TlpLevel::Clear)
    }

    /// Get the TLP:WHITE marking definition (legacy).
    pub fn white() -> MarkingDefinition {
        MarkingDefinition::tlp(TlpLevel::White)
    }

    /// Get the TLP:GREEN marking definition.
    pub fn green() -> MarkingDefinition {
        MarkingDefinition::tlp(TlpLevel::Green)
    }

    /// Get the TLP:AMBER marking definition.
    pub fn amber() -> MarkingDefinition {
        MarkingDefinition::tlp(TlpLevel::Amber)
    }

    /// Get the TLP:RED marking definition.
    pub fn red() -> MarkingDefinition {
        MarkingDefinition::tlp(TlpLevel::Red)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tlp_marking() {
        let marking = MarkingDefinition::tlp(TlpLevel::Amber);
        assert_eq!(marking.type_, "marking-definition");
        assert!(marking.name.as_ref().unwrap().contains("AMBER"));
    }

    #[test]
    fn test_statement_marking() {
        let marking = MarkingDefinition::statement("Copyright 2024 ACME Inc.").unwrap();
        assert_eq!(marking.type_, "marking-definition");
    }

    #[test]
    fn test_granular_marking() {
        let marking_ref: Identifier = "marking-definition--f88d31f6-486f-44da-b317-01333bde0b82"
            .parse()
            .unwrap();
        let gm = GranularMarking::new(marking_ref, vec!["description".to_string()]);
        assert_eq!(gm.selectors.len(), 1);
        assert!(gm.validate().is_ok());
    }

    #[test]
    fn test_granular_marking_with_lang() {
        let gm = GranularMarking::with_lang(vec!["description".to_string()], "en");
        assert!(gm.validate().is_ok());
        assert_eq!(gm.lang, Some("en".to_string()));
        assert!(gm.marking_ref.is_none());
    }

    #[test]
    fn test_granular_marking_mutually_exclusive() {
        // Both lang and marking_ref - should fail
        let marking_ref: Identifier = "marking-definition--f88d31f6-486f-44da-b317-01333bde0b82"
            .parse()
            .unwrap();
        let gm = GranularMarking {
            lang: Some("en".to_string()),
            marking_ref: Some(marking_ref),
            selectors: vec!["description".to_string()],
        };
        assert!(gm.validate().is_err());

        // Neither lang nor marking_ref - should fail
        let gm = GranularMarking {
            lang: None,
            marking_ref: None,
            selectors: vec!["description".to_string()],
        };
        assert!(gm.validate().is_err());
    }
}
