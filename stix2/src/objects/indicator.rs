//! Indicator SDO
//!
//! Indicators contain a pattern that can be used to detect suspicious or
//! malicious cyber activity.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::kill_chain_phase::KillChainPhase;
use crate::core::timestamp::Timestamp;
use crate::impl_sdo_traits;
use crate::patterns::parse_pattern;
use crate::validation::{Constrained, check_timestamp_order_strict};
use crate::vocab::{IndicatorType, PatternType};
use serde::{Deserialize, Serialize};

/// Indicator STIX Domain Object.
///
/// Indicators contain a pattern that can be used to detect suspicious or
/// malicious cyber activity. For example, an Indicator may be used to
/// represent a set of malicious domains.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Indicator {
    /// The type property identifies the type of STIX Object.
    #[serde(rename = "type")]
    pub type_: String,

    /// The id property uniquely identifies this object.
    pub id: Identifier,

    /// Common properties shared by all SDOs.
    #[serde(flatten)]
    pub common: CommonProperties,

    /// A name used to identify the Indicator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// A description that provides more details about the Indicator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A set of categorizations for this indicator.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub indicator_types: Vec<IndicatorType>,

    /// The detection pattern for this Indicator.
    pub pattern: String,

    /// The type of pattern used in this indicator.
    pub pattern_type: PatternType,

    /// The version of the pattern language used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_version: Option<String>,

    /// The time from which this Indicator is considered valid.
    pub valid_from: Timestamp,

    /// The time at which this Indicator should no longer be considered valid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<Timestamp>,

    /// The kill chain phases to which this Indicator corresponds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kill_chain_phases: Vec<KillChainPhase>,
}

impl Indicator {
    /// The STIX type identifier for Indicator.
    pub const TYPE: &'static str = "indicator";

    /// Create a new IndicatorBuilder.
    pub fn builder() -> IndicatorBuilder {
        IndicatorBuilder::new()
    }
}

impl_sdo_traits!(Indicator, "indicator");

impl Constrained for Indicator {
    /// Validate Indicator constraints.
    ///
    /// - `valid_until` must be > `valid_from` (strict inequality)
    /// - If `pattern_type` is STIX, validate the pattern syntax
    fn validate_constraints(&self) -> Result<()> {
        // Check timestamp ordering
        check_timestamp_order_strict(
            Some(&self.valid_from),
            self.valid_until.as_ref(),
            "valid_from",
            "valid_until",
        )?;

        // Validate STIX pattern syntax when pattern_type is "stix"
        if self.pattern_type == PatternType::Stix {
            parse_pattern(&self.pattern)?;
        }

        Ok(())
    }
}

/// Builder for creating Indicator objects.
#[derive(Debug, Default)]
pub struct IndicatorBuilder {
    name: Option<String>,
    description: Option<String>,
    indicator_types: Vec<IndicatorType>,
    pattern: Option<String>,
    pattern_type: Option<PatternType>,
    pattern_version: Option<String>,
    valid_from: Option<Timestamp>,
    valid_until: Option<Timestamp>,
    kill_chain_phases: Vec<KillChainPhase>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(IndicatorBuilder);

impl IndicatorBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add an indicator type.
    pub fn indicator_type(mut self, indicator_type: IndicatorType) -> Self {
        self.indicator_types.push(indicator_type);
        self
    }

    /// Set the pattern (required).
    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set the pattern type (required).
    pub fn pattern_type(mut self, pattern_type: PatternType) -> Self {
        self.pattern_type = Some(pattern_type);
        self
    }

    /// Set the pattern version.
    pub fn pattern_version(mut self, version: impl Into<String>) -> Self {
        self.pattern_version = Some(version.into());
        self
    }

    /// Set the valid_from timestamp (required).
    pub fn valid_from(mut self, valid_from: Timestamp) -> Self {
        self.valid_from = Some(valid_from);
        self
    }

    /// Set the valid_from to now.
    pub fn valid_from_now(mut self) -> Self {
        self.valid_from = Some(Timestamp::now());
        self
    }

    /// Set the valid_until timestamp.
    pub fn valid_until(mut self, valid_until: Timestamp) -> Self {
        self.valid_until = Some(valid_until);
        self
    }

    /// Add a kill chain phase.
    pub fn kill_chain_phase(mut self, phase: KillChainPhase) -> Self {
        self.kill_chain_phases.push(phase);
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

    /// Create an IP address indicator.
    pub fn ip_address(ip: impl Into<String>) -> Self {
        let ip = ip.into();
        Self::new()
            .pattern(format!("[ipv4-addr:value = '{}']", ip))
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .indicator_type(IndicatorType::MaliciousActivity)
    }

    /// Create a domain indicator.
    pub fn domain(domain: impl Into<String>) -> Self {
        let domain = domain.into();
        Self::new()
            .pattern(format!("[domain-name:value = '{}']", domain))
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .indicator_type(IndicatorType::MaliciousActivity)
    }

    /// Create a file hash indicator.
    pub fn file_hash(algorithm: &str, hash: impl Into<String>) -> Self {
        let hash = hash.into();
        Self::new()
            .pattern(format!("[file:hashes.'{}' = '{}']", algorithm, hash))
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .indicator_type(IndicatorType::MaliciousActivity)
    }

    /// Create a URL indicator.
    pub fn url(url: impl Into<String>) -> Self {
        let url = url.into();
        Self::new()
            .pattern(format!("[url:value = '{}']", url))
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .indicator_type(IndicatorType::MaliciousActivity)
    }

    /// Build the Indicator.
    pub fn build(self) -> Result<Indicator> {
        let pattern = self
            .pattern
            .ok_or_else(|| Error::missing_property("pattern"))?;

        let pattern_type = self
            .pattern_type
            .ok_or_else(|| Error::missing_property("pattern_type"))?;

        let valid_from = self
            .valid_from
            .ok_or_else(|| Error::missing_property("valid_from"))?;

        // Auto-set pattern_version to "2.1" when pattern_type is STIX
        let pattern_version = self.pattern_version.or_else(|| {
            if pattern_type == PatternType::Stix {
                Some("2.1".to_string())
            } else {
                None
            }
        });

        let indicator = Indicator {
            type_: Indicator::TYPE.to_string(),
            id: Identifier::new(Indicator::TYPE)?,
            common: self.common,
            name: self.name,
            description: self.description,
            indicator_types: self.indicator_types,
            pattern,
            pattern_type,
            pattern_version,
            valid_from,
            valid_until: self.valid_until,
            kill_chain_phases: self.kill_chain_phases,
        };

        // Validate constraints
        indicator.validate_constraints()?;

        Ok(indicator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_indicator() {
        let indicator = Indicator::builder()
            .name("Malicious File Hash")
            .pattern("[file:hashes.'SHA-256' = 'abc123']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        assert_eq!(indicator.name, Some("Malicious File Hash".to_string()));
        assert_eq!(indicator.type_, "indicator");
        assert_eq!(indicator.pattern_type, PatternType::Stix);
    }

    #[test]
    fn test_ip_address_indicator() {
        let indicator = IndicatorBuilder::ip_address("10.0.0.1")
            .name("Malicious IP")
            .build()
            .unwrap();

        assert!(indicator.pattern.contains("10.0.0.1"));
        assert_eq!(indicator.pattern_type, PatternType::Stix);
    }

    #[test]
    fn test_domain_indicator() {
        let indicator = IndicatorBuilder::domain("malware.example.com")
            .name("Malicious Domain")
            .build()
            .unwrap();

        assert!(indicator.pattern.contains("malware.example.com"));
    }

    #[test]
    fn test_file_hash_indicator() {
        let indicator = IndicatorBuilder::file_hash("SHA-256", "abc123def456")
            .name("Malicious File")
            .build()
            .unwrap();

        assert!(indicator.pattern.contains("SHA-256"));
        assert!(indicator.pattern.contains("abc123def456"));
    }

    #[test]
    fn test_serialization() {
        let indicator = Indicator::builder()
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        let json = serde_json::to_string(&indicator).unwrap();
        let parsed: Indicator = serde_json::from_str(&json).unwrap();
        assert_eq!(indicator.pattern, parsed.pattern);
    }

    #[test]
    fn test_missing_required_fields() {
        // Missing pattern
        let result = Indicator::builder()
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build();
        assert!(result.is_err());

        // Missing pattern_type
        let result = Indicator::builder()
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .valid_from_now()
            .build();
        assert!(result.is_err());

        // Missing valid_from
        let result = Indicator::builder()
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_stix_pattern() {
        // Invalid STIX pattern syntax should fail
        let result = Indicator::builder()
            .pattern("this is not a valid pattern")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build();
        assert!(result.is_err());

        // Missing brackets
        let result = Indicator::builder()
            .pattern("ipv4-addr:value = '10.0.0.1'")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_complex_pattern() {
        // Complex pattern with AND
        let indicator = Indicator::builder()
            .pattern("[ipv4-addr:value = '10.0.0.1'] AND [domain-name:value = 'malware.com']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        assert!(indicator.pattern.contains("AND"));
    }

    #[test]
    fn test_non_stix_pattern_not_validated() {
        // Non-STIX patterns are not validated (e.g., YARA, Snort)
        let indicator = Indicator::builder()
            .pattern("rule malware { strings: $a = \"evil\" condition: $a }")
            .pattern_type(PatternType::Yara)
            .valid_from_now()
            .build();

        // Should succeed even though it's not valid STIX pattern syntax
        assert!(indicator.is_ok());
    }
}
