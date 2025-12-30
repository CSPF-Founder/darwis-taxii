//! Mutex SCO

use super::common::{ScoCommonProperties, generate_sco_id_from_property};
use crate::core::error::Result;
use crate::core::id::Identifier;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Mutex STIX Cyber Observable Object.
///
/// The Mutex object represents the properties of a mutual exclusion (mutex) object.
///
/// # ID Contributing Properties
///
/// - `name`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mutex {
    /// The type property identifies the type of object.
    #[serde(rename = "type")]
    pub type_: String,

    /// Specifies the identifier of the object.
    pub id: Identifier,

    /// The version of the STIX specification used.
    #[serde(default = "default_spec_version")]
    pub spec_version: String,

    /// Specifies the name of the mutex.
    pub name: String,

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

impl Mutex {
    /// The STIX type identifier for Mutex.
    pub const TYPE: &'static str = "mutex";

    /// Create a new Mutex with a deterministic ID based on the name.
    ///
    /// # Arguments
    ///
    /// * `name` - The mutex name
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        let id = generate_sco_id_from_property(Self::TYPE, "name", &json!(name))?;

        Ok(Self {
            type_: Self::TYPE.to_string(),
            id,
            spec_version: default_spec_version(),
            name,
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            defanged: false,
            extensions: IndexMap::new(),
        })
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

impl_sco_traits!(Mutex, "mutex");

impl crate::observables::IdContributing for Mutex {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &["name"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mutex() {
        let mutex = Mutex::new("Global\\MyMutex").unwrap();
        assert_eq!(mutex.type_, "mutex");
        assert_eq!(mutex.name, "Global\\MyMutex");
        assert!(mutex.id.to_string().starts_with("mutex--"));
    }

    #[test]
    fn test_deterministic_id() {
        let mutex1 = Mutex::new("Global\\MyMutex").unwrap();
        let mutex2 = Mutex::new("Global\\MyMutex").unwrap();
        assert_eq!(mutex1.id, mutex2.id);
    }

    #[test]
    fn test_different_values_different_ids() {
        let mutex1 = Mutex::new("Global\\Mutex1").unwrap();
        let mutex2 = Mutex::new("Global\\Mutex2").unwrap();
        assert_ne!(mutex1.id, mutex2.id);
    }

    #[test]
    fn test_serialization() {
        let mutex = Mutex::new("Global\\TestMutex").unwrap();
        let json = serde_json::to_string(&mutex).unwrap();
        assert!(json.contains("\"type\":\"mutex\""));
        assert!(json.contains("\"name\":\"Global\\\\TestMutex\""));

        let parsed: Mutex = serde_json::from_str(&json).unwrap();
        assert_eq!(mutex.name, parsed.name);
        assert_eq!(mutex.id, parsed.id);
    }
}
