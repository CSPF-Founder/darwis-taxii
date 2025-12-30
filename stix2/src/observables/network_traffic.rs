//! Network Traffic SCO

use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use crate::validation::{Constrained, check_timestamp_order};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Network Traffic STIX Cyber Observable Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkTraffic {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(default = "default_spec_version")]
    pub spec_version: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub defanged: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<Timestamp>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_ref: Option<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_ref: Option<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_port: Option<u16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub protocols: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_byte_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_byte_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_packets: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_packets: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipfix: Option<IndexMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_payload_ref: Option<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_payload_ref: Option<Identifier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub encapsulates_refs: Vec<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encapsulated_by_ref: Option<Identifier>,
    /// References to marking definitions that apply to this object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_marking_refs: Vec<Identifier>,
    /// Granular markings for specific properties.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granular_markings: Vec<GranularMarking>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub extensions: IndexMap<String, Value>,
}

fn default_spec_version() -> String {
    "2.1".to_string()
}

impl NetworkTraffic {
    pub const TYPE: &'static str = "network-traffic";

    pub fn new(protocols: Vec<String>) -> Result<Self> {
        Ok(Self {
            type_: Self::TYPE.to_string(),
            id: Identifier::new(Self::TYPE)?,
            spec_version: default_spec_version(),
            defanged: false,
            start: None,
            end: None,
            is_active: false,
            src_ref: None,
            dst_ref: None,
            src_port: None,
            dst_port: None,
            protocols,
            src_byte_count: None,
            dst_byte_count: None,
            src_packets: None,
            dst_packets: None,
            ipfix: None,
            src_payload_ref: None,
            dst_payload_ref: None,
            encapsulates_refs: Vec::new(),
            encapsulated_by_ref: None,
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            extensions: IndexMap::new(),
        })
    }
}

impl_sco_traits!(NetworkTraffic, "network-traffic");

impl crate::observables::IdContributing for NetworkTraffic {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &[
        "start",
        "end",
        "src_ref",
        "dst_ref",
        "src_port",
        "dst_port",
        "protocols",
        "extensions",
    ];
}

impl Constrained for NetworkTraffic {
    /// Validate NetworkTraffic constraints.
    ///
    /// - At least one of `src_ref` or `dst_ref` must be present
    /// - If both `start` and `end` are present, `end` must be >= `start`
    /// - If `end` is present, `is_active` must be false
    fn validate_constraints(&self) -> Result<()> {
        use crate::validation::{check_optional_ref_type, check_refs_type};

        // At least one of src_ref or dst_ref must be present
        if self.src_ref.is_none() && self.dst_ref.is_none() {
            return Err(Error::AtLeastOneRequired(vec![
                "src_ref".to_string(),
                "dst_ref".to_string(),
            ]));
        }

        // end >= start (if both present)
        check_timestamp_order(self.start.as_ref(), self.end.as_ref(), "start", "end")?;

        // Per STIX 2.1 spec: if 'end' is present, 'is_active' must be false
        if self.end.is_some() && self.is_active {
            return Err(Error::InvalidPropertyValue {
                property: "is_active".to_string(),
                message: "'is_active' must be false if 'end' is present".to_string(),
            });
        }

        // Validate reference types
        const ADDR_TYPES: &[&str] = &["ipv4-addr", "ipv6-addr", "mac-addr", "domain-name"];
        check_optional_ref_type(self.src_ref.as_ref(), "src_ref", ADDR_TYPES)?;
        check_optional_ref_type(self.dst_ref.as_ref(), "dst_ref", ADDR_TYPES)?;
        check_optional_ref_type(
            self.src_payload_ref.as_ref(),
            "src_payload_ref",
            &["artifact"],
        )?;
        check_optional_ref_type(
            self.dst_payload_ref.as_ref(),
            "dst_payload_ref",
            &["artifact"],
        )?;
        check_refs_type(
            &self.encapsulates_refs,
            "encapsulates_refs",
            &["network-traffic"],
        )?;
        check_optional_ref_type(
            self.encapsulated_by_ref.as_ref(),
            "encapsulated_by_ref",
            &["network-traffic"],
        )?;

        Ok(())
    }
}
