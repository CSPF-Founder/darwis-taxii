//! Sighting SRO
//!
//! A Sighting denotes the belief that something in CTI was seen.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::impl_sdo_traits;
use crate::validation::{Constrained, check_timestamp_order};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Maximum value for count per STIX 2.1 spec.
const SIGHTING_COUNT_MAX: u64 = 999_999_999;

/// Sighting STIX Relationship Object.
///
/// A Sighting denotes the belief that something in CTI (e.g., an indicator,
/// malware, tool, threat actor, etc.) was seen. Sightings are used to track
/// observations of indicators in real-world data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sighting {
    /// The type property identifies the type of STIX Object.
    #[serde(rename = "type")]
    pub type_: String,

    /// The id property uniquely identifies this object.
    pub id: Identifier,

    /// Common properties shared by all SDOs.
    #[serde(flatten)]
    pub common: CommonProperties,

    /// A description that provides more details about the sighting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The beginning of the time window during which the SDO was sighted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<Timestamp>,

    /// The end of the time window during which the SDO was sighted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<Timestamp>,

    /// The number of times the SDO was sighted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,

    /// An ID reference to the SDO that was sighted.
    pub sighting_of_ref: Identifier,

    /// A list of ID references to the Observed Data objects that contain
    /// the raw cyber data for this Sighting.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_data_refs: Vec<Identifier>,

    /// A list of ID references to the Identity objects that represent
    /// where the sighting occurred.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub where_sighted_refs: Vec<Identifier>,

    /// The summary property indicates whether the Sighting should be
    /// considered summary data.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub summary: bool,

    /// Extensions for this object.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub extensions: IndexMap<String, Value>,
}

impl Sighting {
    /// The STIX type identifier for Sighting.
    pub const TYPE: &'static str = "sighting";

    /// Create a new SightingBuilder.
    pub fn builder() -> SightingBuilder {
        SightingBuilder::new()
    }

    /// Create a simple sighting of an SDO.
    pub fn of(sighting_of_ref: Identifier) -> Result<Self> {
        Self::builder().sighting_of_ref(sighting_of_ref).build()
    }
}

impl_sdo_traits!(Sighting, "sighting");

impl Constrained for Sighting {
    /// Validate Sighting constraints.
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

/// Valid types for where_sighted_refs per STIX 2.1 spec.
const VALID_WHERE_SIGHTED_TYPES: &[&str] = &["identity", "location"];

/// Builder for creating Sighting objects.
#[derive(Debug, Default)]
pub struct SightingBuilder {
    description: Option<String>,
    first_seen: Option<Timestamp>,
    last_seen: Option<Timestamp>,
    count: Option<u64>,
    sighting_of_ref: Option<Identifier>,
    observed_data_refs: Vec<Identifier>,
    where_sighted_refs: Vec<Identifier>,
    summary: bool,
    extensions: IndexMap<String, Value>,
    common: CommonProperties,
}

impl SightingBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
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

    /// Set the count.
    pub fn count(mut self, count: u64) -> Self {
        self.count = Some(count);
        self
    }

    /// Set the sighting_of_ref (required).
    pub fn sighting_of_ref(mut self, sighting_of_ref: Identifier) -> Self {
        self.sighting_of_ref = Some(sighting_of_ref);
        self
    }

    /// Add an observed_data_ref.
    pub fn observed_data_ref(mut self, observed_data_ref: Identifier) -> Self {
        self.observed_data_refs.push(observed_data_ref);
        self
    }

    /// Add a where_sighted_ref.
    pub fn where_sighted_ref(mut self, where_sighted_ref: Identifier) -> Self {
        self.where_sighted_refs.push(where_sighted_ref);
        self
    }

    /// Set the summary flag.
    pub fn summary(mut self, summary: bool) -> Self {
        self.summary = summary;
        self
    }

    /// Set the created_by_ref.
    pub fn created_by_ref(mut self, identity_ref: Identifier) -> Self {
        self.common.created_by_ref = Some(identity_ref);
        self
    }

    /// Set confidence level.
    pub fn confidence(mut self, confidence: u8) -> Self {
        self.common.confidence = Some(confidence.min(100));
        self
    }

    /// Add an extension.
    pub fn extension(mut self, name: impl Into<String>, value: Value) -> Self {
        self.extensions.insert(name.into(), value);
        self
    }

    /// Build the Sighting.
    pub fn build(self) -> Result<Sighting> {
        let sighting_of_ref = self
            .sighting_of_ref
            .ok_or_else(|| Error::missing_property("sighting_of_ref"))?;

        // Validate sighting_of_ref must reference an SDO (not SRO, SCO, or meta objects)
        let sighting_type = sighting_of_ref.object_type();
        let invalid_sighting_types = [
            "relationship",
            "sighting",
            "bundle",
            "language-content",
            "marking-definition",
        ];
        if invalid_sighting_types.contains(&sighting_type) {
            return Err(Error::InvalidPropertyValue {
                property: "sighting_of_ref".to_string(),
                message: format!("'{sighting_type}' is not a valid SDO type for sighting_of_ref"),
            });
        }

        // Validate observed_data_refs must reference observed-data
        for obs_ref in &self.observed_data_refs {
            if obs_ref.object_type() != "observed-data" {
                return Err(Error::InvalidPropertyValue {
                    property: "observed_data_refs".to_string(),
                    message: format!("'{}' is not 'observed-data'", obs_ref.object_type()),
                });
            }
        }

        // Validate where_sighted_refs must reference identity or location
        for where_ref in &self.where_sighted_refs {
            if !VALID_WHERE_SIGHTED_TYPES.contains(&where_ref.object_type()) {
                return Err(Error::InvalidPropertyValue {
                    property: "where_sighted_refs".to_string(),
                    message: format!(
                        "'{}' must be 'identity' or 'location'",
                        where_ref.object_type()
                    ),
                });
            }
        }

        // Validate count range (0-999999999)
        if let Some(count) = self.count
            && count > SIGHTING_COUNT_MAX
        {
            return Err(Error::InvalidPropertyValue {
                property: "count".to_string(),
                message: format!("count must be between 0 and {SIGHTING_COUNT_MAX}"),
            });
        }

        let sighting = Sighting {
            type_: Sighting::TYPE.to_string(),
            id: Identifier::new(Sighting::TYPE)?,
            common: self.common,
            description: self.description,
            first_seen: self.first_seen,
            last_seen: self.last_seen,
            count: self.count,
            sighting_of_ref,
            observed_data_refs: self.observed_data_refs,
            where_sighted_refs: self.where_sighted_refs,
            summary: self.summary,
            extensions: self.extensions,
        };

        // Validate constraints
        sighting.validate_constraints()?;

        Ok(sighting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sighting() {
        let indicator_ref: Identifier = "indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f"
            .parse()
            .unwrap();

        let sighting = Sighting::of(indicator_ref.clone()).unwrap();

        assert_eq!(sighting.type_, "sighting");
        assert_eq!(sighting.sighting_of_ref, indicator_ref);
    }

    #[test]
    fn test_sighting_builder() {
        let indicator_ref: Identifier = "indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f"
            .parse()
            .unwrap();
        let location_ref: Identifier = "identity--31b940d4-6f7f-459a-80ea-9c1f17b5891b"
            .parse()
            .unwrap();

        let sighting = Sighting::builder()
            .sighting_of_ref(indicator_ref)
            .where_sighted_ref(location_ref)
            .count(5)
            .first_seen(Timestamp::now())
            .build()
            .unwrap();

        assert_eq!(sighting.count, Some(5));
        assert_eq!(sighting.where_sighted_refs.len(), 1);
    }

    #[test]
    fn test_serialization() {
        let indicator_ref: Identifier = "indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f"
            .parse()
            .unwrap();

        let sighting = Sighting::of(indicator_ref).unwrap();

        let json = serde_json::to_string(&sighting).unwrap();
        let parsed: Sighting = serde_json::from_str(&json).unwrap();
        assert_eq!(sighting.sighting_of_ref, parsed.sighting_of_ref);
    }
}
