//! Campaign SDO
//!
//! A Campaign is a grouping of adversarial behaviors that describes a set of
//! malicious activities or attacks that occur over a period of time against
//! a specific set of targets.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::impl_sdo_traits;
use crate::validation::{Constrained, check_timestamp_order};
use serde::{Deserialize, Serialize};

/// Campaign STIX Domain Object.
///
/// A Campaign is a grouping of adversarial behaviors that describes a set of
/// malicious activities or attacks (sometimes called waves) that occur over
/// a period of time against a specific set of targets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Campaign {
    /// The type property identifies the type of STIX Object.
    #[serde(rename = "type")]
    pub type_: String,

    /// The id property uniquely identifies this object.
    pub id: Identifier,

    /// Common properties shared by all SDOs.
    #[serde(flatten)]
    pub common: CommonProperties,

    /// A name used to identify the Campaign.
    pub name: String,

    /// A description that provides more details about the Campaign.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Alternative names used to identify this Campaign.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,

    /// The time that this Campaign was first seen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<Timestamp>,

    /// The time that this Campaign was last seen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<Timestamp>,

    /// The Campaign's primary goal, objective, or desired outcome.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objective: Option<String>,
}

impl Campaign {
    /// The STIX type identifier for Campaign.
    pub const TYPE: &'static str = "campaign";

    /// Create a new CampaignBuilder.
    pub fn builder() -> CampaignBuilder {
        CampaignBuilder::new()
    }

    /// Create a new Campaign with the given name.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        Self::builder().name(name).build()
    }
}

impl_sdo_traits!(Campaign, "campaign");

impl Constrained for Campaign {
    /// Validate Campaign constraints.
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

/// Builder for creating Campaign objects.
#[derive(Debug, Default)]
pub struct CampaignBuilder {
    name: Option<String>,
    description: Option<String>,
    aliases: Vec<String>,
    first_seen: Option<Timestamp>,
    last_seen: Option<Timestamp>,
    objective: Option<String>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(CampaignBuilder);

impl CampaignBuilder {
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

    /// Set the objective.
    pub fn objective(mut self, objective: impl Into<String>) -> Self {
        self.objective = Some(objective.into());
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

    /// Build the Campaign.
    pub fn build(self) -> Result<Campaign> {
        let name = self.name.ok_or_else(|| Error::missing_property("name"))?;

        let campaign = Campaign {
            type_: Campaign::TYPE.to_string(),
            id: Identifier::new(Campaign::TYPE)?,
            common: self.common,
            name,
            description: self.description,
            aliases: self.aliases,
            first_seen: self.first_seen,
            last_seen: self.last_seen,
            objective: self.objective,
        };

        // Validate constraints
        campaign.validate_constraints()?;

        Ok(campaign)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_campaign() {
        let campaign = Campaign::builder()
            .name("Operation Aurora")
            .description("A series of cyber attacks")
            .build()
            .unwrap();

        assert_eq!(campaign.name, "Operation Aurora");
        assert_eq!(campaign.type_, "campaign");
    }

    #[test]
    fn test_serialization() {
        let campaign = Campaign::builder()
            .name("Test Campaign")
            .objective("Data exfiltration")
            .build()
            .unwrap();

        let json = serde_json::to_string(&campaign).unwrap();
        let parsed: Campaign = serde_json::from_str(&json).unwrap();
        assert_eq!(campaign.name, parsed.name);
        assert_eq!(campaign.objective, parsed.objective);
    }

    #[test]
    fn test_timestamp_constraint_valid() {
        // first_seen before last_seen - should succeed
        let first = Timestamp::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let last = Timestamp::now();

        let campaign = Campaign::builder()
            .name("Test Campaign")
            .first_seen(first)
            .last_seen(last)
            .build();

        assert!(campaign.is_ok());
    }

    #[test]
    fn test_timestamp_constraint_invalid() {
        // last_seen before first_seen - should fail
        let last = Timestamp::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let first = Timestamp::now();

        let campaign = Campaign::builder()
            .name("Test Campaign")
            .first_seen(first)
            .last_seen(last)
            .build();

        assert!(campaign.is_err());
        let err = campaign.unwrap_err();
        assert!(err.to_string().contains("last_seen"));
    }
}
