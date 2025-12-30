//! Course of Action SDO
//!
//! A Course of Action is an action taken either to prevent an attack or to
//! respond to an attack that is in progress.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::impl_sdo_traits;
use serde::{Deserialize, Serialize};

/// Course of Action STIX Domain Object.
///
/// A Course of Action is an action taken either to prevent an attack or to
/// respond to an attack that is in progress. It may describe technical,
/// automatable responses or higher-level actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CourseOfAction {
    /// The type property identifies the type of STIX Object.
    #[serde(rename = "type")]
    pub type_: String,

    /// The id property uniquely identifies this object.
    pub id: Identifier,

    /// Common properties shared by all SDOs.
    #[serde(flatten)]
    pub common: CommonProperties,

    /// A name used to identify the Course of Action.
    pub name: String,

    /// A description that provides more details about the Course of Action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The type of this action (reserved for future use in STIX 2.1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_type: Option<String>,

    /// The operating system that this course of action applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_execution_envs: Option<Vec<String>>,

    /// A reference to the action content (reserved for future use).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_reference: Option<serde_json::Value>,

    /// The action content itself (reserved for future use).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_bin: Option<String>,
}

impl CourseOfAction {
    /// The STIX type identifier for Course of Action.
    pub const TYPE: &'static str = "course-of-action";

    /// Create a new CourseOfActionBuilder.
    pub fn builder() -> CourseOfActionBuilder {
        CourseOfActionBuilder::new()
    }

    /// Create a new Course of Action with the given name.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        Self::builder().name(name).build()
    }
}

impl_sdo_traits!(CourseOfAction, "course-of-action");

/// Builder for creating CourseOfAction objects.
#[derive(Debug, Default)]
pub struct CourseOfActionBuilder {
    name: Option<String>,
    description: Option<String>,
    action_type: Option<String>,
    os_execution_envs: Option<Vec<String>>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(CourseOfActionBuilder);

impl CourseOfActionBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the name (required).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the action type.
    pub fn action_type(mut self, action_type: impl Into<String>) -> Self {
        self.action_type = Some(action_type.into());
        self
    }

    /// Add an OS execution environment.
    pub fn os_execution_env(mut self, os: impl Into<String>) -> Self {
        self.os_execution_envs
            .get_or_insert_with(Vec::new)
            .push(os.into());
        self
    }

    /// Set the created_by_ref.
    pub fn created_by_ref(mut self, identity_ref: Identifier) -> Self {
        self.common.created_by_ref = Some(identity_ref);
        self
    }

    /// Add a label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.common.labels.push(label.into());
        self
    }

    /// Build the CourseOfAction.
    pub fn build(self) -> Result<CourseOfAction> {
        let name = self.name.ok_or_else(|| Error::missing_property("name"))?;

        Ok(CourseOfAction {
            type_: CourseOfAction::TYPE.to_string(),
            id: Identifier::new(CourseOfAction::TYPE)?,
            common: self.common,
            name,
            description: self.description,
            action_type: self.action_type,
            os_execution_envs: self.os_execution_envs,
            action_reference: None,
            action_bin: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_course_of_action() {
        let coa = CourseOfAction::builder()
            .name("Block Traffic to Malicious IP")
            .description("Add firewall rule to block traffic")
            .build()
            .unwrap();

        assert_eq!(coa.name, "Block Traffic to Malicious IP");
        assert_eq!(coa.type_, "course-of-action");
    }

    #[test]
    fn test_serialization() {
        let coa = CourseOfAction::builder().name("Test CoA").build().unwrap();

        let json = serde_json::to_string(&coa).unwrap();
        let parsed: CourseOfAction = serde_json::from_str(&json).unwrap();
        assert_eq!(coa.name, parsed.name);
    }
}
