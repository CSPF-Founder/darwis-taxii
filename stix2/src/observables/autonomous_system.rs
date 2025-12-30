//! Autonomous System SCO

use super::common::{ScoCommonProperties, generate_sco_id_from_property};
use crate::core::error::Result;
use crate::core::id::Identifier;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Autonomous System STIX Cyber Observable Object.
///
/// The AS object represents an Autonomous System (AS), which is a collection
/// of connected Internet Protocol routing prefixes under the control of one
/// or more network operators.
///
/// # ID Contributing Properties
///
/// - `number`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutonomousSystem {
    /// The type property identifies the type of object.
    #[serde(rename = "type")]
    pub type_: String,

    /// Specifies the identifier of the object.
    pub id: Identifier,

    /// The version of the STIX specification used.
    #[serde(default = "default_spec_version")]
    pub spec_version: String,

    /// Specifies the AS number.
    pub number: u32,

    /// Specifies the name of the AS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Specifies the name of the Regional Internet Registry (RIR).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rir: Option<String>,

    /// References to marking definitions that apply to this object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_marking_refs: Vec<Identifier>,

    /// Granular markings for specific properties.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granular_markings: Vec<GranularMarking>,

    /// Defines whether or not the data contained is defanged.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub defanged: bool,

    /// Extensions for this object.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub extensions: IndexMap<String, Value>,
}

fn default_spec_version() -> String {
    "2.1".to_string()
}

impl AutonomousSystem {
    /// The STIX type identifier for Autonomous System.
    pub const TYPE: &'static str = "autonomous-system";

    /// Create a new Autonomous System with a deterministic ID based on the number.
    ///
    /// # Arguments
    ///
    /// * `number` - The AS number
    pub fn new(number: u32) -> Result<Self> {
        let id = generate_sco_id_from_property(Self::TYPE, "number", &json!(number))?;

        Ok(Self {
            type_: Self::TYPE.to_string(),
            id,
            spec_version: default_spec_version(),
            number,
            name: None,
            rir: None,
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            defanged: false,
            extensions: IndexMap::new(),
        })
    }

    /// Create a new Autonomous System with a name.
    pub fn with_name(number: u32, name: impl Into<String>) -> Result<Self> {
        let mut as_ = Self::new(number)?;
        as_.name = Some(name.into());
        Ok(as_)
    }

    /// Set the name of the AS.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Set the RIR of the AS.
    pub fn set_rir(&mut self, rir: impl Into<String>) {
        self.rir = Some(rir.into());
    }

    /// Add an object marking reference.
    pub fn add_object_marking_ref(&mut self, marking_ref: Identifier) {
        self.object_marking_refs.push(marking_ref);
    }

    /// Add a granular marking.
    pub fn add_granular_marking(&mut self, marking: GranularMarking) {
        self.granular_markings.push(marking);
    }

    /// Apply common SCO properties.
    pub fn with_common_properties(mut self, common: ScoCommonProperties) -> Self {
        self.object_marking_refs = common.object_marking_refs;
        self.granular_markings = common.granular_markings;
        self.extensions = common.extensions;
        self
    }
}

impl_sco_traits!(AutonomousSystem, "autonomous-system");

impl crate::observables::IdContributing for AutonomousSystem {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &["number"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_autonomous_system() {
        let asys = AutonomousSystem::new(15169).unwrap();
        assert_eq!(asys.number, 15169);
        assert_eq!(asys.type_, "autonomous-system");
        assert!(asys.id.to_string().starts_with("autonomous-system--"));
    }

    #[test]
    fn test_deterministic_id() {
        let as1 = AutonomousSystem::new(15169).unwrap();
        let as2 = AutonomousSystem::new(15169).unwrap();
        assert_eq!(as1.id, as2.id);
    }

    #[test]
    fn test_different_values_different_ids() {
        let as1 = AutonomousSystem::new(15169).unwrap();
        let as2 = AutonomousSystem::new(15170).unwrap();
        assert_ne!(as1.id, as2.id);
    }

    #[test]
    fn test_with_name() {
        let asys = AutonomousSystem::with_name(15169, "GOOGLE").unwrap();
        assert_eq!(asys.name, Some("GOOGLE".to_string()));
    }

    #[test]
    fn test_serialization() {
        let asys = AutonomousSystem::with_name(15169, "GOOGLE").unwrap();
        let json = serde_json::to_string(&asys).unwrap();
        assert!(json.contains("\"type\":\"autonomous-system\""));
        assert!(json.contains("\"number\":15169"));
        assert!(json.contains("\"name\":\"GOOGLE\""));

        let parsed: AutonomousSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(asys.number, parsed.number);
        assert_eq!(asys.id, parsed.id);
    }
}
