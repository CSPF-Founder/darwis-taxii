//! Tool SDO
//!
//! Tools are legitimate software that can be used by threat actors to perform attacks.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::kill_chain_phase::KillChainPhase;
use crate::impl_sdo_traits;
use crate::vocab::ToolType;
use serde::{Deserialize, Serialize};

/// Tool STIX Domain Object.
///
/// Tools are legitimate software that can be used by threat actors to perform
/// attacks. Unlike malware, tools are not necessarily malicious themselves,
/// but can be used for malicious purposes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    /// The type property identifies the type of STIX Object.
    #[serde(rename = "type")]
    pub type_: String,

    /// The id property uniquely identifies this object.
    pub id: Identifier,

    /// Common properties shared by all SDOs.
    #[serde(flatten)]
    pub common: CommonProperties,

    /// A name used to identify the Tool.
    pub name: String,

    /// A description that provides more details about the Tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The types of tool.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_types: Vec<ToolType>,

    /// Alternative names for this Tool.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,

    /// The kill chain phases for this Tool.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kill_chain_phases: Vec<KillChainPhase>,

    /// The version of the tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_version: Option<String>,
}

impl Tool {
    /// The STIX type identifier for Tool.
    pub const TYPE: &'static str = "tool";

    /// Create a new ToolBuilder.
    pub fn builder() -> ToolBuilder {
        ToolBuilder::new()
    }

    /// Create a new Tool with the given name.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        Self::builder().name(name).build()
    }
}

impl_sdo_traits!(Tool, "tool");

/// Builder for creating Tool objects.
#[derive(Debug, Default)]
pub struct ToolBuilder {
    name: Option<String>,
    description: Option<String>,
    tool_types: Vec<ToolType>,
    aliases: Vec<String>,
    kill_chain_phases: Vec<KillChainPhase>,
    tool_version: Option<String>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(ToolBuilder);

impl ToolBuilder {
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

    /// Add a tool type.
    pub fn tool_type(mut self, tool_type: ToolType) -> Self {
        self.tool_types.push(tool_type);
        self
    }

    /// Add an alias.
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Add a kill chain phase.
    pub fn kill_chain_phase(mut self, phase: KillChainPhase) -> Self {
        self.kill_chain_phases.push(phase);
        self
    }

    /// Set the tool version.
    pub fn tool_version(mut self, version: impl Into<String>) -> Self {
        self.tool_version = Some(version.into());
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

    /// Build the Tool.
    pub fn build(self) -> Result<Tool> {
        let name = self.name.ok_or_else(|| Error::missing_property("name"))?;

        Ok(Tool {
            type_: Tool::TYPE.to_string(),
            id: Identifier::new(Tool::TYPE)?,
            common: self.common,
            name,
            description: self.description,
            tool_types: self.tool_types,
            aliases: self.aliases,
            kill_chain_phases: self.kill_chain_phases,
            tool_version: self.tool_version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tool() {
        let tool = Tool::builder()
            .name("Mimikatz")
            .tool_type(ToolType::CredentialExploitation)
            .tool_version("2.2.0")
            .build()
            .unwrap();

        assert_eq!(tool.name, "Mimikatz");
        assert_eq!(tool.type_, "tool");
    }

    #[test]
    fn test_serialization() {
        let tool = Tool::builder().name("TestTool").build().unwrap();

        let json = serde_json::to_string(&tool).unwrap();
        let parsed: Tool = serde_json::from_str(&json).unwrap();
        assert_eq!(tool.name, parsed.name);
    }
}
