//! Artifact SCO

use crate::core::common::Hashes;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use crate::validation::Constrained;
use crate::vocab::EncryptionAlgorithm;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Artifact STIX Cyber Observable Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(default = "default_spec_version")]
    pub spec_version: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub defanged: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_bin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Hashes::is_empty")]
    pub hashes: Hashes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_algorithm: Option<EncryptionAlgorithm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decryption_key: Option<String>,
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

fn default_spec_version() -> String {
    "2.1".to_string()
}

impl Artifact {
    pub const TYPE: &'static str = "artifact";

    pub fn from_payload(payload_bin: impl Into<String>) -> Result<Self> {
        Ok(Self {
            type_: Self::TYPE.to_string(),
            id: Identifier::new(Self::TYPE)?,
            spec_version: default_spec_version(),
            defanged: false,
            mime_type: None,
            payload_bin: Some(payload_bin.into()),
            url: None,
            hashes: Hashes::new(),
            encryption_algorithm: None,
            decryption_key: None,
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            extensions: IndexMap::new(),
        })
    }

    pub fn from_url(url: impl Into<String>) -> Result<Self> {
        Ok(Self {
            type_: Self::TYPE.to_string(),
            id: Identifier::new(Self::TYPE)?,
            spec_version: default_spec_version(),
            defanged: false,
            mime_type: None,
            payload_bin: None,
            url: Some(url.into()),
            hashes: Hashes::new(),
            encryption_algorithm: None,
            decryption_key: None,
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            extensions: IndexMap::new(),
        })
    }
}

impl_sco_traits!(Artifact, "artifact");

impl crate::observables::IdContributing for Artifact {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &["hashes", "payload_bin"];
}

impl Constrained for Artifact {
    /// Validate Artifact constraints.
    ///
    /// - `payload_bin` and `url` are mutually exclusive
    /// - If `url` is present, `hashes` must also be present
    fn validate_constraints(&self) -> Result<()> {
        // Check mutually exclusive: payload_bin and url
        if self.payload_bin.is_some() && self.url.is_some() {
            return Err(Error::MutuallyExclusiveProperties(vec![
                "payload_bin".to_string(),
                "url".to_string(),
            ]));
        }

        // If url is present, hashes must be present
        if self.url.is_some() && self.hashes.is_empty() {
            return Err(Error::PropertyDependency {
                dependent: "url".to_string(),
                dependency: "hashes".to_string(),
            });
        }

        Ok(())
    }
}
