//! URL SCO

use super::common::{ScoCommonProperties, generate_sco_id_from_value};
use crate::core::error::Result;
use crate::core::id::Identifier;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// URL STIX Cyber Observable Object.
///
/// The URL object represents the properties of a Uniform Resource Locator (URL).
///
/// # ID Contributing Properties
///
/// - `value`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Url {
    /// The type property identifies the type of object.
    #[serde(rename = "type")]
    pub type_: String,

    /// Specifies the identifier of the object.
    pub id: Identifier,

    /// The version of the STIX specification used.
    #[serde(default = "default_spec_version")]
    pub spec_version: String,

    /// Specifies the URL value.
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

impl Url {
    /// The STIX type identifier for URL.
    pub const TYPE: &'static str = "url";

    /// Create a new URL with a deterministic ID based on the value.
    ///
    /// # Arguments
    ///
    /// * `value` - The URL value (e.g., "https://example.com/path")
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

    /// Create a new defanged URL.
    pub fn defanged(value: impl Into<String>) -> Result<Self> {
        let mut url = Self::new(value)?;
        url.defanged = true;
        Ok(url)
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

impl_sco_traits!(Url, "url");

impl crate::observables::IdContributing for Url {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &["value"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_url() {
        let url = Url::new("https://example.com/malware").unwrap();
        assert_eq!(url.type_, "url");
        assert_eq!(url.value, "https://example.com/malware");
        assert!(url.id.to_string().starts_with("url--"));
    }

    #[test]
    fn test_deterministic_id() {
        let url1 = Url::new("https://example.com/path").unwrap();
        let url2 = Url::new("https://example.com/path").unwrap();
        assert_eq!(url1.id, url2.id);
    }

    #[test]
    fn test_different_values_different_ids() {
        let url1 = Url::new("https://example.com/path1").unwrap();
        let url2 = Url::new("https://example.com/path2").unwrap();
        assert_ne!(url1.id, url2.id);
    }

    #[test]
    fn test_defanged() {
        let url = Url::defanged("hxxps://example[.]com/malware").unwrap();
        assert!(url.defanged);
    }

    #[test]
    fn test_serialization() {
        let url = Url::new("https://example.com/test").unwrap();
        let json = serde_json::to_string(&url).unwrap();
        assert!(json.contains("\"type\":\"url\""));
        assert!(json.contains("\"value\":\"https://example.com/test\""));

        let parsed: Url = serde_json::from_str(&json).unwrap();
        assert_eq!(url.value, parsed.value);
        assert_eq!(url.id, parsed.id);
    }
}
