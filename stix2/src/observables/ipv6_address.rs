//! IPv6 Address SCO

use super::common::{ScoCommonProperties, generate_sco_id_from_value};
use crate::core::error::Result;
use crate::core::id::Identifier;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// IPv6 Address STIX Cyber Observable Object.
///
/// The IPv6 Address object represents one or more IPv6 addresses expressed
/// using CIDR notation.
///
/// # ID Contributing Properties
///
/// - `value`
///
/// # Reference Type Constraints
///
/// - `resolves_to_refs` must contain only `mac-addr` type references
/// - `belongs_to_refs` must contain only `autonomous-system` type references
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IPv6Address {
    /// The type property identifies the type of object.
    #[serde(rename = "type")]
    pub type_: String,

    /// Specifies the identifier of the object.
    pub id: Identifier,

    /// The version of the STIX specification used.
    #[serde(default = "default_spec_version")]
    pub spec_version: String,

    /// Specifies the IPv6 address value.
    pub value: String,

    /// Specifies a list of references to MAC addresses that the IPv6 address resolves to.
    /// References must be of type `mac-addr`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resolves_to_refs: Vec<Identifier>,

    /// Specifies a list of references to Autonomous Systems that the IPv6 address belongs to.
    /// References must be of type `autonomous-system`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub belongs_to_refs: Vec<Identifier>,

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

impl IPv6Address {
    /// The STIX type identifier for IPv6 Address.
    pub const TYPE: &'static str = "ipv6-addr";

    /// Create a new IPv6 Address with a deterministic ID based on the value.
    ///
    /// # Arguments
    ///
    /// * `value` - The IPv6 address value (e.g., "2001:db8::1")
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ip = IPv6Address::new("2001:db8::1")?;
    /// ```
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let id = generate_sco_id_from_value(Self::TYPE, &value)?;

        Ok(Self {
            type_: Self::TYPE.to_string(),
            id,
            spec_version: default_spec_version(),
            value,
            resolves_to_refs: Vec::new(),
            belongs_to_refs: Vec::new(),
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            defanged: false,
            extensions: IndexMap::new(),
        })
    }

    /// Create a new defanged IPv6 Address.
    pub fn defanged(value: impl Into<String>) -> Result<Self> {
        let mut addr = Self::new(value)?;
        addr.defanged = true;
        Ok(addr)
    }

    /// Add a MAC address reference that this IPv6 address resolves to.
    ///
    /// The reference must be of type `mac-addr`.
    pub fn add_resolves_to_ref(&mut self, mac_ref: Identifier) -> Result<()> {
        if mac_ref.object_type() != "mac-addr" {
            return Err(crate::core::error::Error::InvalidType(format!(
                "resolves_to_refs must contain mac-addr references, got: {}",
                mac_ref.object_type()
            )));
        }
        self.resolves_to_refs.push(mac_ref);
        Ok(())
    }

    /// Add an Autonomous System reference that this IPv6 address belongs to.
    ///
    /// The reference must be of type `autonomous-system`.
    pub fn add_belongs_to_ref(&mut self, as_ref: Identifier) -> Result<()> {
        if as_ref.object_type() != "autonomous-system" {
            return Err(crate::core::error::Error::InvalidType(format!(
                "belongs_to_refs must contain autonomous-system references, got: {}",
                as_ref.object_type()
            )));
        }
        self.belongs_to_refs.push(as_ref);
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

impl_sco_traits!(IPv6Address, "ipv6-addr");

impl crate::observables::IdContributing for IPv6Address {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &["value"];
}

impl crate::validation::Constrained for IPv6Address {
    /// Validate IPv6Address constraints.
    ///
    /// - `resolves_to_refs` must reference only `mac-addr`
    /// - `belongs_to_refs` must reference only `autonomous-system`
    fn validate_constraints(&self) -> Result<()> {
        use crate::validation::check_refs_type;

        check_refs_type(&self.resolves_to_refs, "resolves_to_refs", &["mac-addr"])?;
        check_refs_type(
            &self.belongs_to_refs,
            "belongs_to_refs",
            &["autonomous-system"],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ipv6() {
        let ip = IPv6Address::new("2001:db8::1").unwrap();
        assert_eq!(ip.value, "2001:db8::1");
        assert_eq!(ip.type_, "ipv6-addr");
        assert!(ip.id.to_string().starts_with("ipv6-addr--"));
    }

    #[test]
    fn test_deterministic_id() {
        let ip1 = IPv6Address::new("2001:db8::1").unwrap();
        let ip2 = IPv6Address::new("2001:db8::1").unwrap();
        // Same value should produce same ID
        assert_eq!(ip1.id, ip2.id);
    }

    #[test]
    fn test_different_values_different_ids() {
        let ip1 = IPv6Address::new("2001:db8::1").unwrap();
        let ip2 = IPv6Address::new("2001:db8::2").unwrap();
        // Different values should produce different IDs
        assert_ne!(ip1.id, ip2.id);
    }

    #[test]
    fn test_defanged() {
        let ip = IPv6Address::defanged("2001:db8::1").unwrap();
        assert!(ip.defanged);
    }

    #[test]
    fn test_resolves_to_refs_validation() {
        let mut ip = IPv6Address::new("2001:db8::1").unwrap();
        let mac_ref: Identifier = "mac-addr--12345678-1234-1234-1234-123456789abc"
            .parse()
            .unwrap();
        assert!(ip.add_resolves_to_ref(mac_ref).is_ok());

        let bad_ref: Identifier = "ipv4-addr--12345678-1234-1234-1234-123456789abc"
            .parse()
            .unwrap();
        assert!(ip.add_resolves_to_ref(bad_ref).is_err());
    }

    #[test]
    fn test_belongs_to_refs_validation() {
        let mut ip = IPv6Address::new("2001:db8::1").unwrap();
        let as_ref: Identifier = "autonomous-system--12345678-1234-1234-1234-123456789abc"
            .parse()
            .unwrap();
        assert!(ip.add_belongs_to_ref(as_ref).is_ok());

        let bad_ref: Identifier = "ipv4-addr--12345678-1234-1234-1234-123456789abc"
            .parse()
            .unwrap();
        assert!(ip.add_belongs_to_ref(bad_ref).is_err());
    }

    #[test]
    fn test_serialization() {
        let ip = IPv6Address::new("2001:db8::1").unwrap();
        let json = serde_json::to_string(&ip).unwrap();
        assert!(json.contains("\"type\":\"ipv6-addr\""));
        assert!(json.contains("\"value\":\"2001:db8::1\""));

        // Deserialize and verify
        let parsed: IPv6Address = serde_json::from_str(&json).unwrap();
        assert_eq!(ip.value, parsed.value);
        assert_eq!(ip.id, parsed.id);
    }
}
