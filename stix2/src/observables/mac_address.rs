//! MAC Address SCO

use super::common::{ScoCommonProperties, generate_sco_id_from_value};
use crate::core::error::Result;
use crate::core::id::Identifier;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MAC Address STIX Cyber Observable Object.
///
/// The MAC Address object represents a single Media Access Control (MAC) address.
///
/// # ID Contributing Properties
///
/// - `value`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacAddress {
    /// The type property identifies the type of object.
    #[serde(rename = "type")]
    pub type_: String,

    /// Specifies the identifier of the object.
    pub id: Identifier,

    /// The version of the STIX specification used.
    #[serde(default = "default_spec_version")]
    pub spec_version: String,

    /// Specifies the MAC address value.
    pub value: String,

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

impl MacAddress {
    /// The STIX type identifier for MAC Address.
    pub const TYPE: &'static str = "mac-addr";

    /// Create a new MAC Address with a deterministic ID based on the value.
    ///
    /// # Arguments
    ///
    /// * `value` - The MAC address value (e.g., "00:00:5e:00:53:af")
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let id = generate_sco_id_from_value(Self::TYPE, &value)?;

        Ok(Self {
            type_: Self::TYPE.to_string(),
            id,
            spec_version: default_spec_version(),
            value,
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            defanged: false,
            extensions: IndexMap::new(),
        })
    }

    /// Create a new defanged MAC Address.
    pub fn defanged(value: impl Into<String>) -> Result<Self> {
        let mut addr = Self::new(value)?;
        addr.defanged = true;
        Ok(addr)
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

impl_sco_traits!(MacAddress, "mac-addr");

impl crate::observables::IdContributing for MacAddress {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &["value"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mac() {
        let mac = MacAddress::new("00:00:5e:00:53:af").unwrap();
        assert_eq!(mac.value, "00:00:5e:00:53:af");
        assert_eq!(mac.type_, "mac-addr");
        assert!(mac.id.to_string().starts_with("mac-addr--"));
    }

    #[test]
    fn test_deterministic_id() {
        let mac1 = MacAddress::new("00:00:5e:00:53:af").unwrap();
        let mac2 = MacAddress::new("00:00:5e:00:53:af").unwrap();
        assert_eq!(mac1.id, mac2.id);
    }

    #[test]
    fn test_different_values_different_ids() {
        let mac1 = MacAddress::new("00:00:5e:00:53:af").unwrap();
        let mac2 = MacAddress::new("00:00:5e:00:53:00").unwrap();
        assert_ne!(mac1.id, mac2.id);
    }

    #[test]
    fn test_serialization() {
        let mac = MacAddress::new("00:00:5e:00:53:af").unwrap();
        let json = serde_json::to_string(&mac).unwrap();
        assert!(json.contains("\"type\":\"mac-addr\""));
        assert!(json.contains("\"value\":\"00:00:5e:00:53:af\""));

        let parsed: MacAddress = serde_json::from_str(&json).unwrap();
        assert_eq!(mac.value, parsed.value);
        assert_eq!(mac.id, parsed.id);
    }
}
