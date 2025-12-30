//! X.509 Certificate SCO

use crate::core::common::Hashes;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::extensions::X509V3ExtensionsType;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use crate::validation::Constrained;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// X.509 Certificate STIX Cyber Observable Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct X509Certificate {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(default = "default_spec_version")]
    pub spec_version: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub defanged: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_self_signed: bool,
    #[serde(default, skip_serializing_if = "Hashes::is_empty")]
    pub hashes: Hashes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_algorithm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_not_before: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_not_after: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_public_key_algorithm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_public_key_modulus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_public_key_exponent: Option<u64>,
    /// X.509 v3 extension properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x509_v3_extensions: Option<X509V3ExtensionsType>,
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

impl X509Certificate {
    pub const TYPE: &'static str = "x509-certificate";

    pub fn new() -> Result<Self> {
        Ok(Self {
            type_: Self::TYPE.to_string(),
            id: Identifier::new(Self::TYPE)?,
            spec_version: default_spec_version(),
            defanged: false,
            is_self_signed: false,
            hashes: Hashes::new(),
            version: None,
            serial_number: None,
            signature_algorithm: None,
            issuer: None,
            validity_not_before: None,
            validity_not_after: None,
            subject: None,
            subject_public_key_algorithm: None,
            subject_public_key_modulus: None,
            subject_public_key_exponent: None,
            x509_v3_extensions: None,
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            extensions: IndexMap::new(),
        })
    }
}

impl_sco_traits!(X509Certificate, "x509-certificate");

impl crate::observables::IdContributing for X509Certificate {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &["hashes", "serial_number"];
}

impl Constrained for X509Certificate {
    /// Validate X509Certificate constraints.
    ///
    /// - At least one property (besides type, id, spec_version, defanged) must be present
    fn validate_constraints(&self) -> Result<()> {
        // Check if at least one optional property is present
        let has_content = self.is_self_signed
            || !self.hashes.is_empty()
            || self.version.is_some()
            || self.serial_number.is_some()
            || self.signature_algorithm.is_some()
            || self.issuer.is_some()
            || self.validity_not_before.is_some()
            || self.validity_not_after.is_some()
            || self.subject.is_some()
            || self.subject_public_key_algorithm.is_some()
            || self.subject_public_key_modulus.is_some()
            || self.subject_public_key_exponent.is_some()
            || self.x509_v3_extensions.is_some();

        if !has_content {
            return Err(Error::AtLeastOneRequired(vec![
                "is_self_signed".to_string(),
                "hashes".to_string(),
                "version".to_string(),
                "serial_number".to_string(),
                "signature_algorithm".to_string(),
                "issuer".to_string(),
                "validity_not_before".to_string(),
                "validity_not_after".to_string(),
                "subject".to_string(),
                "subject_public_key_algorithm".to_string(),
                "subject_public_key_modulus".to_string(),
                "subject_public_key_exponent".to_string(),
                "x509_v3_extensions".to_string(),
            ]));
        }

        Ok(())
    }
}
