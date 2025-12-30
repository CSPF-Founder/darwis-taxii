//! Directory SCO

use crate::core::error::Result;
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Directory STIX Cyber Observable Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Directory {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(default = "default_spec_version")]
    pub spec_version: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub defanged: bool,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_enc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ctime: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtime: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atime: Option<Timestamp>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contains_refs: Vec<Identifier>,
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

impl Directory {
    pub const TYPE: &'static str = "directory";

    pub fn new(path: impl Into<String>) -> Result<Self> {
        Ok(Self {
            type_: Self::TYPE.to_string(),
            id: Identifier::new(Self::TYPE)?,
            spec_version: default_spec_version(),
            defanged: false,
            path: path.into(),
            path_enc: None,
            ctime: None,
            mtime: None,
            atime: None,
            contains_refs: Vec::new(),
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            extensions: IndexMap::new(),
        })
    }
}

impl_sco_traits!(Directory, "directory");

impl crate::observables::IdContributing for Directory {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &["path"];
}

impl crate::validation::Constrained for Directory {
    /// Validate Directory constraints.
    ///
    /// - `contains_refs` must reference only `file` or `directory` types
    fn validate_constraints(&self) -> Result<()> {
        use crate::validation::check_refs_type;

        check_refs_type(&self.contains_refs, "contains_refs", &["file", "directory"])?;

        Ok(())
    }
}
