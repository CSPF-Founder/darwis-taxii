//! Windows Registry Key SCO

use crate::core::error::Result;
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use crate::vocab::WindowsRegistryDatatype;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Windows Registry Value Type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowsRegistryValueType {
    /// The name of the registry value. This should be optional per STIX spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<WindowsRegistryDatatype>,
}

/// Windows Registry Key STIX Cyber Observable Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowsRegistryKey {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(default = "default_spec_version")]
    pub spec_version: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub defanged: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<WindowsRegistryValueType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_time: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_user_ref: Option<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_subkeys: Option<u32>,
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

impl WindowsRegistryKey {
    pub const TYPE: &'static str = "windows-registry-key";

    pub fn new(key: impl Into<String>) -> Result<Self> {
        Ok(Self {
            type_: Self::TYPE.to_string(),
            id: Identifier::new(Self::TYPE)?,
            spec_version: default_spec_version(),
            defanged: false,
            key: Some(key.into()),
            values: Vec::new(),
            modified_time: None,
            creator_user_ref: None,
            number_of_subkeys: None,
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            extensions: IndexMap::new(),
        })
    }
}

impl_sco_traits!(WindowsRegistryKey, "windows-registry-key");

impl crate::observables::IdContributing for WindowsRegistryKey {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &["key", "values"];
}

impl crate::validation::Constrained for WindowsRegistryKey {
    /// Validate WindowsRegistryKey constraints.
    fn validate_constraints(&self) -> crate::core::error::Result<()> {
        use crate::validation::check_optional_ref_type;

        // Validate creator_user_ref references a user-account
        check_optional_ref_type(
            self.creator_user_ref.as_ref(),
            "creator_user_ref",
            &["user-account"],
        )?;

        Ok(())
    }
}
