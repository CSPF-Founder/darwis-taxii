//! Infrastructure SDO (STIX 2.1)
//!
//! Infrastructure represents resources used or controlled by adversaries.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::kill_chain_phase::KillChainPhase;
use crate::core::timestamp::Timestamp;
use crate::impl_sdo_traits;
use crate::validation::{Constrained, check_timestamp_order};
use crate::vocab::InfrastructureType;
use serde::{Deserialize, Serialize};

/// Infrastructure STIX Domain Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Infrastructure {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(flatten)]
    pub common: CommonProperties,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub infrastructure_types: Vec<InfrastructureType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kill_chain_phases: Vec<KillChainPhase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<Timestamp>,
}

impl Infrastructure {
    pub const TYPE: &'static str = "infrastructure";

    pub fn builder() -> InfrastructureBuilder {
        InfrastructureBuilder::new()
    }

    pub fn new(name: impl Into<String>) -> Result<Self> {
        Self::builder().name(name).build()
    }
}

impl_sdo_traits!(Infrastructure, "infrastructure");

impl Constrained for Infrastructure {
    /// Validate Infrastructure constraints.
    ///
    /// - `last_seen` must be >= `first_seen`
    fn validate_constraints(&self) -> Result<()> {
        check_timestamp_order(
            self.first_seen.as_ref(),
            self.last_seen.as_ref(),
            "first_seen",
            "last_seen",
        )
    }
}

#[derive(Debug, Default)]
pub struct InfrastructureBuilder {
    name: Option<String>,
    description: Option<String>,
    infrastructure_types: Vec<InfrastructureType>,
    aliases: Vec<String>,
    kill_chain_phases: Vec<KillChainPhase>,
    first_seen: Option<Timestamp>,
    last_seen: Option<Timestamp>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(InfrastructureBuilder);

impl InfrastructureBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn infrastructure_type(mut self, it: InfrastructureType) -> Self {
        self.infrastructure_types.push(it);
        self
    }

    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    pub fn kill_chain_phase(mut self, phase: KillChainPhase) -> Self {
        self.kill_chain_phases.push(phase);
        self
    }

    pub fn first_seen(mut self, first_seen: Timestamp) -> Self {
        self.first_seen = Some(first_seen);
        self
    }

    pub fn last_seen(mut self, last_seen: Timestamp) -> Self {
        self.last_seen = Some(last_seen);
        self
    }

    pub fn created_by_ref(mut self, identity_ref: Identifier) -> Self {
        self.common.created_by_ref = Some(identity_ref);
        self
    }

    pub fn build(self) -> Result<Infrastructure> {
        let name = self.name.ok_or_else(|| Error::missing_property("name"))?;

        let infrastructure = Infrastructure {
            type_: Infrastructure::TYPE.to_string(),
            id: Identifier::new(Infrastructure::TYPE)?,
            common: self.common,
            name,
            description: self.description,
            infrastructure_types: self.infrastructure_types,
            aliases: self.aliases,
            kill_chain_phases: self.kill_chain_phases,
            first_seen: self.first_seen,
            last_seen: self.last_seen,
        };

        // Validate constraints
        infrastructure.validate_constraints()?;

        Ok(infrastructure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_infrastructure() {
        let infra = Infrastructure::builder()
            .name("C2 Server")
            .infrastructure_type(InfrastructureType::CommandAndControl)
            .build()
            .unwrap();

        assert_eq!(infra.type_, "infrastructure");
    }
}
