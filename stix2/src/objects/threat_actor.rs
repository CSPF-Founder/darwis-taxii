//! Threat Actor SDO
//!
//! Threat Actors are actual individuals, groups, or organizations believed
//! to be operating with malicious intent.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::impl_sdo_traits;
use crate::validation::{Constrained, check_timestamp_order};
use crate::vocab::{
    AttackMotivation, AttackResourceLevel, ThreatActorRole, ThreatActorSophistication,
    ThreatActorType,
};
use serde::{Deserialize, Serialize};

/// Threat Actor STIX Domain Object.
///
/// Threat Actors are actual individuals, groups, or organizations believed
/// to be operating with malicious intent. Threat Actors are characterized
/// by their goals, motivations, resources, and sophistication.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreatActor {
    /// The type property identifies the type of STIX Object.
    #[serde(rename = "type")]
    pub type_: String,

    /// The id property uniquely identifies this object.
    pub id: Identifier,

    /// Common properties shared by all SDOs.
    #[serde(flatten)]
    pub common: CommonProperties,

    /// A name used to identify this Threat Actor.
    pub name: String,

    /// A description that provides more details about the Threat Actor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The types of this threat actor.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub threat_actor_types: Vec<ThreatActorType>,

    /// Alternative names for this Threat Actor.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,

    /// The time the threat actor was first seen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<Timestamp>,

    /// The time the threat actor was last seen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<Timestamp>,

    /// The roles this threat actor plays.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<ThreatActorRole>,

    /// The goals of this threat actor.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub goals: Vec<String>,

    /// The sophistication level of this threat actor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sophistication: Option<ThreatActorSophistication>,

    /// The organizational/individual resource level of this threat actor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_level: Option<AttackResourceLevel>,

    /// The primary motivation of this threat actor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_motivation: Option<AttackMotivation>,

    /// The secondary motivations of this threat actor.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary_motivations: Vec<AttackMotivation>,

    /// The personal motivations of this threat actor.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub personal_motivations: Vec<AttackMotivation>,
}

impl ThreatActor {
    /// The STIX type identifier for Threat Actor.
    pub const TYPE: &'static str = "threat-actor";

    /// Create a new ThreatActorBuilder.
    pub fn builder() -> ThreatActorBuilder {
        ThreatActorBuilder::new()
    }

    /// Create a new Threat Actor with the given name.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        Self::builder().name(name).build()
    }
}

impl_sdo_traits!(ThreatActor, "threat-actor");

impl Constrained for ThreatActor {
    /// Validate ThreatActor constraints.
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

/// Builder for creating ThreatActor objects.
#[derive(Debug, Default)]
pub struct ThreatActorBuilder {
    name: Option<String>,
    description: Option<String>,
    threat_actor_types: Vec<ThreatActorType>,
    aliases: Vec<String>,
    first_seen: Option<Timestamp>,
    last_seen: Option<Timestamp>,
    roles: Vec<ThreatActorRole>,
    goals: Vec<String>,
    sophistication: Option<ThreatActorSophistication>,
    resource_level: Option<AttackResourceLevel>,
    primary_motivation: Option<AttackMotivation>,
    secondary_motivations: Vec<AttackMotivation>,
    personal_motivations: Vec<AttackMotivation>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(ThreatActorBuilder);

impl ThreatActorBuilder {
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

    /// Add a threat actor type.
    pub fn threat_actor_type(mut self, ta_type: ThreatActorType) -> Self {
        self.threat_actor_types.push(ta_type);
        self
    }

    /// Add an alias.
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Set the first seen timestamp.
    pub fn first_seen(mut self, first_seen: Timestamp) -> Self {
        self.first_seen = Some(first_seen);
        self
    }

    /// Set the last seen timestamp.
    pub fn last_seen(mut self, last_seen: Timestamp) -> Self {
        self.last_seen = Some(last_seen);
        self
    }

    /// Add a role.
    pub fn role(mut self, role: ThreatActorRole) -> Self {
        self.roles.push(role);
        self
    }

    /// Add a goal.
    pub fn goal(mut self, goal: impl Into<String>) -> Self {
        self.goals.push(goal.into());
        self
    }

    /// Set the sophistication level.
    pub fn sophistication(mut self, sophistication: ThreatActorSophistication) -> Self {
        self.sophistication = Some(sophistication);
        self
    }

    /// Set the resource level.
    pub fn resource_level(mut self, resource_level: AttackResourceLevel) -> Self {
        self.resource_level = Some(resource_level);
        self
    }

    /// Set the primary motivation.
    pub fn primary_motivation(mut self, motivation: AttackMotivation) -> Self {
        self.primary_motivation = Some(motivation);
        self
    }

    /// Add a secondary motivation.
    pub fn secondary_motivation(mut self, motivation: AttackMotivation) -> Self {
        self.secondary_motivations.push(motivation);
        self
    }

    /// Add a personal motivation.
    pub fn personal_motivation(mut self, motivation: AttackMotivation) -> Self {
        self.personal_motivations.push(motivation);
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

    /// Build the ThreatActor.
    pub fn build(self) -> Result<ThreatActor> {
        let name = self.name.ok_or_else(|| Error::missing_property("name"))?;

        let threat_actor = ThreatActor {
            type_: ThreatActor::TYPE.to_string(),
            id: Identifier::new(ThreatActor::TYPE)?,
            common: self.common,
            name,
            description: self.description,
            threat_actor_types: self.threat_actor_types,
            aliases: self.aliases,
            first_seen: self.first_seen,
            last_seen: self.last_seen,
            roles: self.roles,
            goals: self.goals,
            sophistication: self.sophistication,
            resource_level: self.resource_level,
            primary_motivation: self.primary_motivation,
            secondary_motivations: self.secondary_motivations,
            personal_motivations: self.personal_motivations,
        };

        // Validate constraints
        threat_actor.validate_constraints()?;

        Ok(threat_actor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_threat_actor() {
        let ta = ThreatActor::builder()
            .name("APT28")
            .threat_actor_type(ThreatActorType::NationState)
            .sophistication(ThreatActorSophistication::Advanced)
            .primary_motivation(AttackMotivation::Ideology)
            .build()
            .unwrap();

        assert_eq!(ta.name, "APT28");
        assert_eq!(ta.type_, "threat-actor");
    }

    #[test]
    fn test_threat_actor_with_aliases() {
        let ta = ThreatActor::builder()
            .name("APT28")
            .alias("Fancy Bear")
            .alias("Sofacy")
            .alias("Pawn Storm")
            .build()
            .unwrap();

        assert_eq!(ta.aliases.len(), 3);
    }

    #[test]
    fn test_serialization() {
        let ta = ThreatActor::builder().name("Test Actor").build().unwrap();

        let json = serde_json::to_string(&ta).unwrap();
        let parsed: ThreatActor = serde_json::from_str(&json).unwrap();
        assert_eq!(ta.name, parsed.name);
    }
}
