//! Attack Pattern SDO
//!
//! Attack Patterns are a type of TTP that describe ways that adversaries attempt
//! to compromise targets.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::external_reference::ExternalReference;
use crate::core::id::Identifier;
use crate::core::kill_chain_phase::KillChainPhase;
use crate::impl_sdo_traits;
use serde::{Deserialize, Serialize};

/// Attack Pattern STIX Domain Object.
///
/// Attack Patterns are a type of TTP that describe ways that adversaries
/// attempt to compromise targets. They are used to help categorize attacks,
/// generalize specific attacks to the patterns that they follow, and provide
/// detailed information about how attacks are performed.
///
/// # Example
///
/// ```rust,no_run
/// use stix2::objects::AttackPattern;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let attack = AttackPattern::builder()
///         .name("Spear Phishing")
///         .description("Targeted phishing attack using personalized content")
///         .build()?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttackPattern {
    /// The type property identifies the type of STIX Object.
    #[serde(rename = "type")]
    pub type_: String,

    /// The id property uniquely identifies this object.
    pub id: Identifier,

    /// Common properties shared by all SDOs.
    #[serde(flatten)]
    pub common: CommonProperties,

    /// A name used to identify the Attack Pattern.
    pub name: String,

    /// A description that provides more details about the attack pattern.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Alternative names for this attack pattern.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,

    /// The list of Kill Chain Phases for which this Attack Pattern is used.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kill_chain_phases: Vec<KillChainPhase>,
}

impl AttackPattern {
    /// The STIX type identifier for Attack Pattern.
    pub const TYPE: &'static str = "attack-pattern";

    /// Create a new AttackPatternBuilder.
    pub fn builder() -> AttackPatternBuilder {
        AttackPatternBuilder::new()
    }

    /// Create a new Attack Pattern with the given name.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        Self::builder().name(name).build()
    }
}

impl_sdo_traits!(AttackPattern, "attack-pattern");

/// Builder for creating AttackPattern objects.
#[derive(Debug, Default)]
pub struct AttackPatternBuilder {
    name: Option<String>,
    description: Option<String>,
    aliases: Vec<String>,
    kill_chain_phases: Vec<KillChainPhase>,
    common: CommonProperties,
    external_references: Vec<ExternalReference>,
}

// Implement common builder methods
crate::impl_common_builder_methods!(AttackPatternBuilder);

impl AttackPatternBuilder {
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

    /// Add an alias.
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Set all aliases.
    pub fn aliases(mut self, aliases: Vec<String>) -> Self {
        self.aliases = aliases;
        self
    }

    /// Add a kill chain phase.
    pub fn kill_chain_phase(mut self, phase: KillChainPhase) -> Self {
        self.kill_chain_phases.push(phase);
        self
    }

    /// Set all kill chain phases.
    pub fn kill_chain_phases(mut self, phases: Vec<KillChainPhase>) -> Self {
        self.kill_chain_phases = phases;
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

    /// Set confidence level.
    pub fn confidence(mut self, confidence: u8) -> Self {
        self.common.confidence = Some(confidence.min(100));
        self
    }

    /// Add a MITRE ATT&CK reference.
    pub fn mitre_attack(self, technique_id: impl Into<String>) -> Self {
        self.external_reference(ExternalReference::mitre_attack(technique_id))
    }

    /// Add a CAPEC reference.
    pub fn capec(self, capec_id: impl Into<String>) -> Self {
        self.external_reference(ExternalReference::capec(capec_id))
    }

    /// Build the AttackPattern.
    pub fn build(self) -> Result<AttackPattern> {
        let name = self.name.ok_or_else(|| Error::missing_property("name"))?;

        let mut common = self.common;
        common.external_references = self.external_references;

        Ok(AttackPattern {
            type_: AttackPattern::TYPE.to_string(),
            id: Identifier::new(AttackPattern::TYPE)?,
            common,
            name,
            description: self.description,
            aliases: self.aliases,
            kill_chain_phases: self.kill_chain_phases,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_attack_pattern() {
        let ap = AttackPattern::builder()
            .name("Spear Phishing")
            .description("Targeted phishing attack")
            .build()
            .unwrap();

        assert_eq!(ap.name, "Spear Phishing");
        assert_eq!(ap.type_, "attack-pattern");
        assert!(ap.id.to_string().starts_with("attack-pattern--"));
    }

    #[test]
    fn test_attack_pattern_with_kill_chain() {
        use crate::core::kill_chain_phase::mitre_attack;

        let ap = AttackPattern::builder()
            .name("Phishing")
            .kill_chain_phase(mitre_attack::initial_access())
            .build()
            .unwrap();

        assert_eq!(ap.kill_chain_phases.len(), 1);
    }

    #[test]
    fn test_serialization() {
        let ap = AttackPattern::builder()
            .name("Test Attack")
            .build()
            .unwrap();

        let json = serde_json::to_string(&ap).unwrap();
        assert!(json.contains("\"type\":\"attack-pattern\""));
        assert!(json.contains("\"name\":\"Test Attack\""));

        let parsed: AttackPattern = serde_json::from_str(&json).unwrap();
        assert_eq!(ap.name, parsed.name);
    }

    #[test]
    fn test_missing_name_error() {
        let result = AttackPattern::builder().build();
        assert!(result.is_err());
    }
}
