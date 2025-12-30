//! Intrusion Set SDO
//!
//! An Intrusion Set is a grouped set of adversarial behaviors and resources
//! with common properties believed to be orchestrated by a single organization.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::impl_sdo_traits;
use crate::validation::{Constrained, check_timestamp_order};
use crate::vocab::{AttackMotivation, AttackResourceLevel};
use serde::{Deserialize, Serialize};

/// Intrusion Set STIX Domain Object.
///
/// An Intrusion Set is a grouped set of adversarial behaviors and resources
/// with common properties that is believed to be orchestrated by a single
/// organization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntrusionSet {
    /// The type property identifies the type of STIX Object.
    #[serde(rename = "type")]
    pub type_: String,

    /// The id property uniquely identifies this object.
    pub id: Identifier,

    /// Common properties shared by all SDOs.
    #[serde(flatten)]
    pub common: CommonProperties,

    /// A name used to identify this Intrusion Set.
    pub name: String,

    /// A description that provides more details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Alternative names for this Intrusion Set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,

    /// The time this Intrusion Set was first seen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<Timestamp>,

    /// The time this Intrusion Set was last seen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<Timestamp>,

    /// High-level goals of this Intrusion Set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub goals: Vec<String>,

    /// The resource level of this Intrusion Set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_level: Option<AttackResourceLevel>,

    /// The primary motivation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_motivation: Option<AttackMotivation>,

    /// Secondary motivations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary_motivations: Vec<AttackMotivation>,
}

impl IntrusionSet {
    /// The STIX type identifier.
    pub const TYPE: &'static str = "intrusion-set";

    /// Create a new builder.
    pub fn builder() -> IntrusionSetBuilder {
        IntrusionSetBuilder::new()
    }

    /// Create a new Intrusion Set with the given name.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        Self::builder().name(name).build()
    }
}

impl_sdo_traits!(IntrusionSet, "intrusion-set");

impl Constrained for IntrusionSet {
    /// Validate IntrusionSet constraints.
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

/// Builder for creating IntrusionSet objects.
#[derive(Debug, Default)]
pub struct IntrusionSetBuilder {
    name: Option<String>,
    description: Option<String>,
    aliases: Vec<String>,
    first_seen: Option<Timestamp>,
    last_seen: Option<Timestamp>,
    goals: Vec<String>,
    resource_level: Option<AttackResourceLevel>,
    primary_motivation: Option<AttackMotivation>,
    secondary_motivations: Vec<AttackMotivation>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(IntrusionSetBuilder);

impl IntrusionSetBuilder {
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

    /// Add a goal.
    pub fn goal(mut self, goal: impl Into<String>) -> Self {
        self.goals.push(goal.into());
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

    /// Set the created_by_ref.
    pub fn created_by_ref(mut self, identity_ref: Identifier) -> Self {
        self.common.created_by_ref = Some(identity_ref);
        self
    }

    /// Build the IntrusionSet.
    pub fn build(self) -> Result<IntrusionSet> {
        let name = self.name.ok_or_else(|| Error::missing_property("name"))?;

        let intrusion_set = IntrusionSet {
            type_: IntrusionSet::TYPE.to_string(),
            id: Identifier::new(IntrusionSet::TYPE)?,
            common: self.common,
            name,
            description: self.description,
            aliases: self.aliases,
            first_seen: self.first_seen,
            last_seen: self.last_seen,
            goals: self.goals,
            resource_level: self.resource_level,
            primary_motivation: self.primary_motivation,
            secondary_motivations: self.secondary_motivations,
        };

        // Validate constraints
        intrusion_set.validate_constraints()?;

        Ok(intrusion_set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_intrusion_set() {
        let is = IntrusionSet::builder()
            .name("APT28")
            .alias("Fancy Bear")
            .primary_motivation(AttackMotivation::Ideology)
            .build()
            .unwrap();

        assert_eq!(is.name, "APT28");
        assert_eq!(is.type_, "intrusion-set");
    }
}
