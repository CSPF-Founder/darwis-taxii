//! Domain Name SCO

use super::common::{ScoCommonProperties, generate_sco_id_from_value};
use crate::core::error::Result;
use crate::core::id::Identifier;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Domain Name STIX Cyber Observable Object.
///
/// The Domain Name object represents a network domain name.
///
/// # ID Contributing Properties
///
/// - `value`
///
/// # Reference Type Constraints
///
/// - `resolves_to_refs` must contain only `ipv4-addr`, `ipv6-addr`, or `domain-name` type references
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainName {
    /// The type property identifies the type of object.
    #[serde(rename = "type")]
    pub type_: String,

    /// Specifies the identifier of the object.
    pub id: Identifier,

    /// The version of the STIX specification used.
    #[serde(default = "default_spec_version")]
    pub spec_version: String,

    /// Specifies the domain name value.
    pub value: String,

    /// Specifies a list of references to IP addresses or domain names that this domain resolves to.
    /// References must be of type `ipv4-addr`, `ipv6-addr`, or `domain-name`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resolves_to_refs: Vec<Identifier>,

    /// References to marking definitions that apply to this object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_marking_refs: Vec<Identifier>,

    /// Granular markings for specific properties.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granular_markings: Vec<GranularMarking>,

    /// Defines whether or not the data contained is defanged.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub defanged: bool,

    /// Extensions for this object.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub extensions: IndexMap<String, Value>,
}

fn default_spec_version() -> String {
    "2.1".to_string()
}

impl DomainName {
    /// The STIX type identifier for Domain Name.
    pub const TYPE: &'static str = "domain-name";

    /// Valid types for resolves_to_refs.
    const VALID_RESOLVES_TO_TYPES: &'static [&'static str] =
        &["ipv4-addr", "ipv6-addr", "domain-name"];

    /// Create a new Domain Name with a deterministic ID based on the value.
    ///
    /// # Arguments
    ///
    /// * `value` - The domain name value (e.g., "example.com")
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let id = generate_sco_id_from_value(Self::TYPE, &value)?;

        Ok(Self {
            type_: Self::TYPE.to_string(),
            id,
            spec_version: default_spec_version(),
            value,
            resolves_to_refs: Vec::new(),
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            defanged: false,
            extensions: IndexMap::new(),
        })
    }

    /// Create a new defanged Domain Name.
    pub fn defanged(value: impl Into<String>) -> Result<Self> {
        let mut domain = Self::new(value)?;
        domain.defanged = true;
        Ok(domain)
    }

    /// Add a reference to an IP address or domain name that this domain resolves to.
    ///
    /// The reference must be of type `ipv4-addr`, `ipv6-addr`, or `domain-name`.
    pub fn add_resolves_to_ref(&mut self, ref_: Identifier) -> Result<()> {
        if !Self::VALID_RESOLVES_TO_TYPES.contains(&ref_.object_type()) {
            return Err(crate::core::error::Error::InvalidType(format!(
                "resolves_to_refs must contain ipv4-addr, ipv6-addr, or domain-name references, got: {}",
                ref_.object_type()
            )));
        }
        self.resolves_to_refs.push(ref_);
        Ok(())
    }

    /// Add an object marking reference.
    pub fn add_object_marking_ref(&mut self, marking_ref: Identifier) {
        self.object_marking_refs.push(marking_ref);
    }

    /// Add a granular marking.
    pub fn add_granular_marking(&mut self, marking: GranularMarking) {
        self.granular_markings.push(marking);
    }

    /// Apply common SCO properties.
    pub fn with_common_properties(mut self, common: ScoCommonProperties) -> Self {
        self.object_marking_refs = common.object_marking_refs;
        self.granular_markings = common.granular_markings;
        self.extensions = common.extensions;
        self
    }
}

impl_sco_traits!(DomainName, "domain-name");

impl crate::observables::IdContributing for DomainName {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &["value"];
}

impl crate::validation::Constrained for DomainName {
    /// Validate DomainName constraints.
    ///
    /// - `resolves_to_refs` must reference only `ipv4-addr`, `ipv6-addr`, or `domain-name`
    fn validate_constraints(&self) -> crate::core::error::Result<()> {
        use crate::validation::check_refs_type;

        check_refs_type(
            &self.resolves_to_refs,
            "resolves_to_refs",
            Self::VALID_RESOLVES_TO_TYPES,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_domain() {
        let domain = DomainName::new("example.com").unwrap();
        assert_eq!(domain.value, "example.com");
        assert_eq!(domain.type_, "domain-name");
        assert!(domain.id.to_string().starts_with("domain-name--"));
    }

    #[test]
    fn test_deterministic_id() {
        let domain1 = DomainName::new("example.com").unwrap();
        let domain2 = DomainName::new("example.com").unwrap();
        assert_eq!(domain1.id, domain2.id);
    }

    #[test]
    fn test_different_values_different_ids() {
        let domain1 = DomainName::new("example.com").unwrap();
        let domain2 = DomainName::new("example.org").unwrap();
        assert_ne!(domain1.id, domain2.id);
    }

    #[test]
    fn test_resolves_to_refs_validation() {
        let mut domain = DomainName::new("example.com").unwrap();

        let ipv4_ref: Identifier = "ipv4-addr--12345678-1234-1234-1234-123456789abc"
            .parse()
            .unwrap();
        assert!(domain.add_resolves_to_ref(ipv4_ref).is_ok());

        let ipv6_ref: Identifier = "ipv6-addr--12345678-1234-1234-1234-123456789abc"
            .parse()
            .unwrap();
        assert!(domain.add_resolves_to_ref(ipv6_ref).is_ok());

        let domain_ref: Identifier = "domain-name--12345678-1234-1234-1234-123456789abc"
            .parse()
            .unwrap();
        assert!(domain.add_resolves_to_ref(domain_ref).is_ok());

        let bad_ref: Identifier = "mac-addr--12345678-1234-1234-1234-123456789abc"
            .parse()
            .unwrap();
        assert!(domain.add_resolves_to_ref(bad_ref).is_err());
    }

    #[test]
    fn test_serialization() {
        let domain = DomainName::new("example.com").unwrap();
        let json = serde_json::to_string(&domain).unwrap();
        assert!(json.contains("\"type\":\"domain-name\""));
        assert!(json.contains("\"value\":\"example.com\""));

        let parsed: DomainName = serde_json::from_str(&json).unwrap();
        assert_eq!(domain.value, parsed.value);
        assert_eq!(domain.id, parsed.id);
    }
}
