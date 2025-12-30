//! Incident SDO (STIX 2.1)
//!
//! An Incident is a discrete occurrence of a particular kind of security event.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::kill_chain_phase::KillChainPhase;
use crate::impl_sdo_traits;
use serde::{Deserialize, Serialize};

/// Incident STIX Domain Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Incident {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(flatten)]
    pub common: CommonProperties,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The list of Kill Chain Phases for which this Incident is used.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kill_chain_phases: Vec<KillChainPhase>,
}

impl Incident {
    pub const TYPE: &'static str = "incident";

    pub fn builder() -> IncidentBuilder {
        IncidentBuilder::new()
    }

    pub fn new(name: impl Into<String>) -> Result<Self> {
        Self::builder().name(name).build()
    }
}

impl_sdo_traits!(Incident, "incident");

#[derive(Debug, Default)]
pub struct IncidentBuilder {
    name: Option<String>,
    description: Option<String>,
    kill_chain_phases: Vec<KillChainPhase>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(IncidentBuilder);

impl IncidentBuilder {
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

    /// Add a kill chain phase.
    pub fn kill_chain_phase(mut self, phase: KillChainPhase) -> Self {
        self.kill_chain_phases.push(phase);
        self
    }

    pub fn created_by_ref(mut self, identity_ref: Identifier) -> Self {
        self.common.created_by_ref = Some(identity_ref);
        self
    }

    pub fn build(self) -> Result<Incident> {
        let name = self.name.ok_or_else(|| Error::missing_property("name"))?;

        Ok(Incident {
            type_: Incident::TYPE.to_string(),
            id: Identifier::new(Incident::TYPE)?,
            common: self.common,
            name,
            description: self.description,
            kill_chain_phases: self.kill_chain_phases,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::kill_chain_phase::mitre_attack;

    #[test]
    fn test_create_incident() {
        let incident = Incident::new("Data Breach 2023").unwrap();
        assert_eq!(incident.type_, "incident");
    }

    #[test]
    fn test_incident_with_kill_chain_phases() {
        let incident = Incident::builder()
            .name("Advanced Persistent Threat Incident")
            .kill_chain_phase(mitre_attack::initial_access())
            .kill_chain_phase(mitre_attack::persistence())
            .build()
            .unwrap();

        assert_eq!(incident.kill_chain_phases.len(), 2);
    }
}
