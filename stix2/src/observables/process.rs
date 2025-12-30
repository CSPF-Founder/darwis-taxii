//! Process SCO

use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use crate::validation::Constrained;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Process STIX Cyber Observable Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Process {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(default = "default_spec_version")]
    pub spec_version: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub defanged: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_time: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_line: Option<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub environment_variables: IndexMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub opened_connection_refs: Vec<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_user_ref: Option<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_ref: Option<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_ref: Option<Identifier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub child_refs: Vec<Identifier>,
    /// References to marking definitions that apply to this object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_marking_refs: Vec<Identifier>,
    /// Granular markings for specific properties.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granular_markings: Vec<GranularMarking>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub extensions: IndexMap<String, Value>,
}

fn default_spec_version() -> String {
    "2.1".to_string()
}

impl Process {
    pub const TYPE: &'static str = "process";

    pub fn new() -> Result<Self> {
        Ok(Self {
            type_: Self::TYPE.to_string(),
            id: Identifier::new(Self::TYPE)?,
            spec_version: default_spec_version(),
            defanged: false,
            is_hidden: false,
            pid: None,
            created_time: None,
            cwd: None,
            command_line: None,
            environment_variables: IndexMap::new(),
            opened_connection_refs: Vec::new(),
            creator_user_ref: None,
            image_ref: None,
            parent_ref: None,
            child_refs: Vec::new(),
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            extensions: IndexMap::new(),
        })
    }
}

impl_sco_traits!(Process, "process");

impl crate::observables::IdContributing for Process {
    // Process uses random UUID - no ID contributing properties
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &[];
}

impl Constrained for Process {
    /// Validate Process constraints.
    ///
    /// - At least one property (besides type, id, spec_version, defanged, extensions) must be present
    /// - pid must be non-negative if present
    fn validate_constraints(&self) -> Result<()> {
        use crate::validation::{check_non_negative, check_optional_ref_type, check_refs_type};

        // Check if at least one optional property is present
        let has_content = self.is_hidden
            || self.pid.is_some()
            || self.created_time.is_some()
            || self.cwd.is_some()
            || self.command_line.is_some()
            || !self.environment_variables.is_empty()
            || !self.opened_connection_refs.is_empty()
            || self.creator_user_ref.is_some()
            || self.image_ref.is_some()
            || self.parent_ref.is_some()
            || !self.child_refs.is_empty();

        if !has_content {
            return Err(Error::AtLeastOneRequired(vec![
                "is_hidden".to_string(),
                "pid".to_string(),
                "created_time".to_string(),
                "cwd".to_string(),
                "command_line".to_string(),
                "environment_variables".to_string(),
                "opened_connection_refs".to_string(),
                "creator_user_ref".to_string(),
                "image_ref".to_string(),
                "parent_ref".to_string(),
                "child_refs".to_string(),
            ]));
        }

        // Validate pid is non-negative
        if let Some(pid) = self.pid {
            check_non_negative(pid, "pid")?;
        }

        // Validate reference types
        check_refs_type(
            &self.opened_connection_refs,
            "opened_connection_refs",
            &["network-traffic"],
        )?;
        check_optional_ref_type(
            self.creator_user_ref.as_ref(),
            "creator_user_ref",
            &["user-account"],
        )?;
        check_optional_ref_type(self.image_ref.as_ref(), "image_ref", &["file"])?;
        check_optional_ref_type(self.parent_ref.as_ref(), "parent_ref", &["process"])?;
        check_refs_type(&self.child_refs, "child_refs", &["process"])?;

        Ok(())
    }
}
