//! Email Address SCO

use crate::core::error::Result;
use crate::core::id::Identifier;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Email Address STIX Cyber Observable Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailAddress {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(default = "default_spec_version")]
    pub spec_version: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub defanged: bool,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub belongs_to_ref: Option<Identifier>,
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

impl EmailAddress {
    pub const TYPE: &'static str = "email-addr";

    pub fn new(value: impl Into<String>) -> Result<Self> {
        Ok(Self {
            type_: Self::TYPE.to_string(),
            id: Identifier::new(Self::TYPE)?,
            spec_version: default_spec_version(),
            defanged: false,
            value: value.into(),
            display_name: None,
            belongs_to_ref: None,
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            extensions: IndexMap::new(),
        })
    }
}

impl_sco_traits!(EmailAddress, "email-addr");

impl crate::observables::IdContributing for EmailAddress {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &["value"];
}

impl crate::validation::Constrained for EmailAddress {
    /// Validate EmailAddress constraints.
    ///
    /// - `belongs_to_ref` must reference a `user-account`
    fn validate_constraints(&self) -> Result<()> {
        use crate::validation::check_optional_ref_type;

        check_optional_ref_type(
            self.belongs_to_ref.as_ref(),
            "belongs_to_ref",
            &["user-account"],
        )?;

        Ok(())
    }
}
