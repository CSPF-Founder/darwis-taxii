//! Unified STIX object enum.
//!
//! This module provides a comprehensive enum that can represent any STIX object,
//! enabling heterogeneous collections and dynamic dispatch.

use crate::core::id::Identifier;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// Unified enum representing any STIX object.
///
/// This enum allows storing different STIX object types in the same collection,
/// such as in a Bundle. It supports serialization/deserialization with automatic
/// type detection based on the "type" field.
#[derive(Debug, Clone, PartialEq)]
pub enum StixObject {
    // STIX Domain Objects (SDOs)
    /// Attack Pattern SDO
    AttackPattern(crate::objects::AttackPattern),
    /// Campaign SDO
    Campaign(crate::objects::Campaign),
    /// Course of Action SDO
    CourseOfAction(crate::objects::CourseOfAction),
    /// Grouping SDO (STIX 2.1)
    Grouping(crate::objects::Grouping),
    /// Identity SDO
    Identity(crate::objects::Identity),
    /// Incident SDO (STIX 2.1)
    Incident(crate::objects::Incident),
    /// Indicator SDO
    Indicator(crate::objects::Indicator),
    /// Infrastructure SDO (STIX 2.1)
    Infrastructure(crate::objects::Infrastructure),
    /// Intrusion Set SDO
    IntrusionSet(crate::objects::IntrusionSet),
    /// Location SDO (STIX 2.1)
    Location(crate::objects::Location),
    /// Malware SDO
    Malware(crate::objects::Malware),
    /// Malware Analysis SDO (STIX 2.1)
    MalwareAnalysis(crate::objects::MalwareAnalysis),
    /// Note SDO (STIX 2.1)
    Note(crate::objects::Note),
    /// Observed Data SDO
    ObservedData(crate::objects::ObservedData),
    /// Opinion SDO (STIX 2.1)
    Opinion(crate::objects::Opinion),
    /// Report SDO
    Report(crate::objects::Report),
    /// Threat Actor SDO
    ThreatActor(crate::objects::ThreatActor),
    /// Tool SDO
    Tool(crate::objects::Tool),
    /// Vulnerability SDO
    Vulnerability(crate::objects::Vulnerability),

    // STIX Relationship Objects (SROs)
    /// Relationship SRO
    Relationship(crate::relationship::Relationship),
    /// Sighting SRO
    Sighting(crate::relationship::Sighting),

    // STIX Cyber Observable Objects (SCOs)
    /// Artifact SCO
    Artifact(crate::observables::Artifact),
    /// Autonomous System SCO
    AutonomousSystem(crate::observables::AutonomousSystem),
    /// Directory SCO
    Directory(crate::observables::Directory),
    /// Domain Name SCO
    DomainName(crate::observables::DomainName),
    /// Email Address SCO
    EmailAddress(crate::observables::EmailAddress),
    /// Email Message SCO
    EmailMessage(crate::observables::EmailMessage),
    /// File SCO
    File(crate::observables::File),
    /// IPv4 Address SCO
    IPv4Address(crate::observables::IPv4Address),
    /// IPv6 Address SCO
    IPv6Address(crate::observables::IPv6Address),
    /// MAC Address SCO
    MacAddress(crate::observables::MacAddress),
    /// Mutex SCO
    Mutex(crate::observables::Mutex),
    /// Network Traffic SCO
    NetworkTraffic(crate::observables::NetworkTraffic),
    /// Process SCO
    Process(crate::observables::Process),
    /// Software SCO
    Software(crate::observables::Software),
    /// URL SCO
    Url(crate::observables::Url),
    /// User Account SCO
    UserAccount(crate::observables::UserAccount),
    /// Windows Registry Key SCO
    WindowsRegistryKey(crate::observables::WindowsRegistryKey),
    /// X.509 Certificate SCO
    X509Certificate(crate::observables::X509Certificate),

    // Marking Definitions
    /// Marking Definition
    MarkingDefinition(crate::markings::MarkingDefinition),

    // Language Content (STIX 2.1)
    /// Language Content object
    LanguageContent(crate::objects::LanguageContent),

    /// Custom or unknown object type (stored as raw JSON)
    Custom(CustomObject),
}

/// A custom or unknown STIX object stored as raw JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomObject {
    /// The object type.
    #[serde(rename = "type")]
    pub type_: String,

    /// The object ID.
    pub id: Identifier,

    /// The raw JSON value.
    #[serde(flatten)]
    pub properties: Value,
}

impl StixObject {
    /// Get the type name of this object.
    pub fn type_name(&self) -> &str {
        match self {
            StixObject::AttackPattern(_) => "attack-pattern",
            StixObject::Campaign(_) => "campaign",
            StixObject::CourseOfAction(_) => "course-of-action",
            StixObject::Grouping(_) => "grouping",
            StixObject::Identity(_) => "identity",
            StixObject::Incident(_) => "incident",
            StixObject::Indicator(_) => "indicator",
            StixObject::Infrastructure(_) => "infrastructure",
            StixObject::IntrusionSet(_) => "intrusion-set",
            StixObject::Location(_) => "location",
            StixObject::Malware(_) => "malware",
            StixObject::MalwareAnalysis(_) => "malware-analysis",
            StixObject::Note(_) => "note",
            StixObject::ObservedData(_) => "observed-data",
            StixObject::Opinion(_) => "opinion",
            StixObject::Report(_) => "report",
            StixObject::ThreatActor(_) => "threat-actor",
            StixObject::Tool(_) => "tool",
            StixObject::Vulnerability(_) => "vulnerability",
            StixObject::Relationship(_) => "relationship",
            StixObject::Sighting(_) => "sighting",
            StixObject::Artifact(_) => "artifact",
            StixObject::AutonomousSystem(_) => "autonomous-system",
            StixObject::Directory(_) => "directory",
            StixObject::DomainName(_) => "domain-name",
            StixObject::EmailAddress(_) => "email-addr",
            StixObject::EmailMessage(_) => "email-message",
            StixObject::File(_) => "file",
            StixObject::IPv4Address(_) => "ipv4-addr",
            StixObject::IPv6Address(_) => "ipv6-addr",
            StixObject::MacAddress(_) => "mac-addr",
            StixObject::Mutex(_) => "mutex",
            StixObject::NetworkTraffic(_) => "network-traffic",
            StixObject::Process(_) => "process",
            StixObject::Software(_) => "software",
            StixObject::Url(_) => "url",
            StixObject::UserAccount(_) => "user-account",
            StixObject::WindowsRegistryKey(_) => "windows-registry-key",
            StixObject::X509Certificate(_) => "x509-certificate",
            StixObject::MarkingDefinition(_) => "marking-definition",
            StixObject::LanguageContent(_) => "language-content",
            StixObject::Custom(c) => &c.type_,
        }
    }

    /// Get the object's identifier.
    pub fn id(&self) -> &Identifier {
        match self {
            StixObject::AttackPattern(o) => &o.id,
            StixObject::Campaign(o) => &o.id,
            StixObject::CourseOfAction(o) => &o.id,
            StixObject::Grouping(o) => &o.id,
            StixObject::Identity(o) => &o.id,
            StixObject::Incident(o) => &o.id,
            StixObject::Indicator(o) => &o.id,
            StixObject::Infrastructure(o) => &o.id,
            StixObject::IntrusionSet(o) => &o.id,
            StixObject::Location(o) => &o.id,
            StixObject::Malware(o) => &o.id,
            StixObject::MalwareAnalysis(o) => &o.id,
            StixObject::Note(o) => &o.id,
            StixObject::ObservedData(o) => &o.id,
            StixObject::Opinion(o) => &o.id,
            StixObject::Report(o) => &o.id,
            StixObject::ThreatActor(o) => &o.id,
            StixObject::Tool(o) => &o.id,
            StixObject::Vulnerability(o) => &o.id,
            StixObject::Relationship(o) => &o.id,
            StixObject::Sighting(o) => &o.id,
            StixObject::Artifact(o) => &o.id,
            StixObject::AutonomousSystem(o) => &o.id,
            StixObject::Directory(o) => &o.id,
            StixObject::DomainName(o) => &o.id,
            StixObject::EmailAddress(o) => &o.id,
            StixObject::EmailMessage(o) => &o.id,
            StixObject::File(o) => &o.id,
            StixObject::IPv4Address(o) => &o.id,
            StixObject::IPv6Address(o) => &o.id,
            StixObject::MacAddress(o) => &o.id,
            StixObject::Mutex(o) => &o.id,
            StixObject::NetworkTraffic(o) => &o.id,
            StixObject::Process(o) => &o.id,
            StixObject::Software(o) => &o.id,
            StixObject::Url(o) => &o.id,
            StixObject::UserAccount(o) => &o.id,
            StixObject::WindowsRegistryKey(o) => &o.id,
            StixObject::X509Certificate(o) => &o.id,
            StixObject::MarkingDefinition(o) => &o.id,
            StixObject::LanguageContent(o) => &o.id,
            StixObject::Custom(o) => &o.id,
        }
    }

    /// Get the modified timestamp if available.
    pub fn modified(&self) -> Option<DateTime<Utc>> {
        match self {
            StixObject::AttackPattern(o) => Some(o.common.modified.datetime()),
            StixObject::Campaign(o) => Some(o.common.modified.datetime()),
            StixObject::CourseOfAction(o) => Some(o.common.modified.datetime()),
            StixObject::Grouping(o) => Some(o.common.modified.datetime()),
            StixObject::Identity(o) => Some(o.common.modified.datetime()),
            StixObject::Incident(o) => Some(o.common.modified.datetime()),
            StixObject::Indicator(o) => Some(o.common.modified.datetime()),
            StixObject::Infrastructure(o) => Some(o.common.modified.datetime()),
            StixObject::IntrusionSet(o) => Some(o.common.modified.datetime()),
            StixObject::Location(o) => Some(o.common.modified.datetime()),
            StixObject::Malware(o) => Some(o.common.modified.datetime()),
            StixObject::MalwareAnalysis(o) => Some(o.common.modified.datetime()),
            StixObject::Note(o) => Some(o.common.modified.datetime()),
            StixObject::ObservedData(o) => Some(o.common.modified.datetime()),
            StixObject::Opinion(o) => Some(o.common.modified.datetime()),
            StixObject::Report(o) => Some(o.common.modified.datetime()),
            StixObject::ThreatActor(o) => Some(o.common.modified.datetime()),
            StixObject::Tool(o) => Some(o.common.modified.datetime()),
            StixObject::Vulnerability(o) => Some(o.common.modified.datetime()),
            StixObject::Relationship(o) => Some(o.common.modified.datetime()),
            StixObject::Sighting(o) => Some(o.common.modified.datetime()),
            StixObject::MarkingDefinition(o) => Some(o.created.datetime()),
            StixObject::LanguageContent(o) => Some(o.common.modified.datetime()),
            // SCOs don't have modified timestamps
            _ => None,
        }
    }

    /// Check if this is a domain object (SDO).
    pub fn is_domain_object(&self) -> bool {
        matches!(
            self,
            StixObject::AttackPattern(_)
                | StixObject::Campaign(_)
                | StixObject::CourseOfAction(_)
                | StixObject::Grouping(_)
                | StixObject::Identity(_)
                | StixObject::Incident(_)
                | StixObject::Indicator(_)
                | StixObject::Infrastructure(_)
                | StixObject::IntrusionSet(_)
                | StixObject::Location(_)
                | StixObject::Malware(_)
                | StixObject::MalwareAnalysis(_)
                | StixObject::Note(_)
                | StixObject::ObservedData(_)
                | StixObject::Opinion(_)
                | StixObject::Report(_)
                | StixObject::ThreatActor(_)
                | StixObject::Tool(_)
                | StixObject::Vulnerability(_)
        )
    }

    /// Check if this is a relationship object (SRO).
    pub fn is_relationship_object(&self) -> bool {
        matches!(self, StixObject::Relationship(_) | StixObject::Sighting(_))
    }

    /// Check if this is a cyber observable object (SCO).
    pub fn is_cyber_observable(&self) -> bool {
        matches!(
            self,
            StixObject::Artifact(_)
                | StixObject::AutonomousSystem(_)
                | StixObject::Directory(_)
                | StixObject::DomainName(_)
                | StixObject::EmailAddress(_)
                | StixObject::EmailMessage(_)
                | StixObject::File(_)
                | StixObject::IPv4Address(_)
                | StixObject::IPv6Address(_)
                | StixObject::MacAddress(_)
                | StixObject::Mutex(_)
                | StixObject::NetworkTraffic(_)
                | StixObject::Process(_)
                | StixObject::Software(_)
                | StixObject::Url(_)
                | StixObject::UserAccount(_)
                | StixObject::WindowsRegistryKey(_)
                | StixObject::X509Certificate(_)
        )
    }

    /// Check if this is a marking definition.
    pub fn is_marking_definition(&self) -> bool {
        matches!(self, StixObject::MarkingDefinition(_))
    }

    /// Try to convert to a specific type.
    pub fn as_indicator(&self) -> Option<&crate::objects::Indicator> {
        match self {
            StixObject::Indicator(i) => Some(i),
            _ => None,
        }
    }

    /// Try to convert to a specific type.
    pub fn as_malware(&self) -> Option<&crate::objects::Malware> {
        match self {
            StixObject::Malware(m) => Some(m),
            _ => None,
        }
    }

    /// Try to convert to a relationship.
    pub fn as_relationship(&self) -> Option<&crate::relationship::Relationship> {
        match self {
            StixObject::Relationship(r) => Some(r),
            _ => None,
        }
    }
}

impl Serialize for StixObject {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            StixObject::AttackPattern(o) => o.serialize(serializer),
            StixObject::Campaign(o) => o.serialize(serializer),
            StixObject::CourseOfAction(o) => o.serialize(serializer),
            StixObject::Grouping(o) => o.serialize(serializer),
            StixObject::Identity(o) => o.serialize(serializer),
            StixObject::Incident(o) => o.serialize(serializer),
            StixObject::Indicator(o) => o.serialize(serializer),
            StixObject::Infrastructure(o) => o.serialize(serializer),
            StixObject::IntrusionSet(o) => o.serialize(serializer),
            StixObject::Location(o) => o.serialize(serializer),
            StixObject::Malware(o) => o.serialize(serializer),
            StixObject::MalwareAnalysis(o) => o.serialize(serializer),
            StixObject::Note(o) => o.serialize(serializer),
            StixObject::ObservedData(o) => o.serialize(serializer),
            StixObject::Opinion(o) => o.serialize(serializer),
            StixObject::Report(o) => o.serialize(serializer),
            StixObject::ThreatActor(o) => o.serialize(serializer),
            StixObject::Tool(o) => o.serialize(serializer),
            StixObject::Vulnerability(o) => o.serialize(serializer),
            StixObject::Relationship(o) => o.serialize(serializer),
            StixObject::Sighting(o) => o.serialize(serializer),
            StixObject::Artifact(o) => o.serialize(serializer),
            StixObject::AutonomousSystem(o) => o.serialize(serializer),
            StixObject::Directory(o) => o.serialize(serializer),
            StixObject::DomainName(o) => o.serialize(serializer),
            StixObject::EmailAddress(o) => o.serialize(serializer),
            StixObject::EmailMessage(o) => o.serialize(serializer),
            StixObject::File(o) => o.serialize(serializer),
            StixObject::IPv4Address(o) => o.serialize(serializer),
            StixObject::IPv6Address(o) => o.serialize(serializer),
            StixObject::MacAddress(o) => o.serialize(serializer),
            StixObject::Mutex(o) => o.serialize(serializer),
            StixObject::NetworkTraffic(o) => o.serialize(serializer),
            StixObject::Process(o) => o.serialize(serializer),
            StixObject::Software(o) => o.serialize(serializer),
            StixObject::Url(o) => o.serialize(serializer),
            StixObject::UserAccount(o) => o.serialize(serializer),
            StixObject::WindowsRegistryKey(o) => o.serialize(serializer),
            StixObject::X509Certificate(o) => o.serialize(serializer),
            StixObject::MarkingDefinition(o) => o.serialize(serializer),
            StixObject::LanguageContent(o) => o.serialize(serializer),
            StixObject::Custom(o) => o.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for StixObject {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let type_str = value
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde::de::Error::custom("missing 'type' field"))?;

        let result = match type_str {
            "attack-pattern" => serde_json::from_value(value).map(StixObject::AttackPattern),
            "campaign" => serde_json::from_value(value).map(StixObject::Campaign),
            "course-of-action" => serde_json::from_value(value).map(StixObject::CourseOfAction),
            "grouping" => serde_json::from_value(value).map(StixObject::Grouping),
            "identity" => serde_json::from_value(value).map(StixObject::Identity),
            "incident" => serde_json::from_value(value).map(StixObject::Incident),
            "indicator" => serde_json::from_value(value).map(StixObject::Indicator),
            "infrastructure" => serde_json::from_value(value).map(StixObject::Infrastructure),
            "intrusion-set" => serde_json::from_value(value).map(StixObject::IntrusionSet),
            "location" => serde_json::from_value(value).map(StixObject::Location),
            "malware" => serde_json::from_value(value).map(StixObject::Malware),
            "malware-analysis" => serde_json::from_value(value).map(StixObject::MalwareAnalysis),
            "note" => serde_json::from_value(value).map(StixObject::Note),
            "observed-data" => serde_json::from_value(value).map(StixObject::ObservedData),
            "opinion" => serde_json::from_value(value).map(StixObject::Opinion),
            "report" => serde_json::from_value(value).map(StixObject::Report),
            "threat-actor" => serde_json::from_value(value).map(StixObject::ThreatActor),
            "tool" => serde_json::from_value(value).map(StixObject::Tool),
            "vulnerability" => serde_json::from_value(value).map(StixObject::Vulnerability),
            "relationship" => serde_json::from_value(value).map(StixObject::Relationship),
            "sighting" => serde_json::from_value(value).map(StixObject::Sighting),
            "artifact" => serde_json::from_value(value).map(StixObject::Artifact),
            "autonomous-system" => serde_json::from_value(value).map(StixObject::AutonomousSystem),
            "directory" => serde_json::from_value(value).map(StixObject::Directory),
            "domain-name" => serde_json::from_value(value).map(StixObject::DomainName),
            "email-addr" => serde_json::from_value(value).map(StixObject::EmailAddress),
            "email-message" => serde_json::from_value(value).map(StixObject::EmailMessage),
            "file" => serde_json::from_value(value).map(StixObject::File),
            "ipv4-addr" => serde_json::from_value(value).map(StixObject::IPv4Address),
            "ipv6-addr" => serde_json::from_value(value).map(StixObject::IPv6Address),
            "mac-addr" => serde_json::from_value(value).map(StixObject::MacAddress),
            "mutex" => serde_json::from_value(value).map(StixObject::Mutex),
            "network-traffic" => serde_json::from_value(value).map(StixObject::NetworkTraffic),
            "process" => serde_json::from_value(value).map(StixObject::Process),
            "software" => serde_json::from_value(value).map(StixObject::Software),
            "url" => serde_json::from_value(value).map(StixObject::Url),
            "user-account" => serde_json::from_value(value).map(StixObject::UserAccount),
            "windows-registry-key" => {
                serde_json::from_value(value).map(StixObject::WindowsRegistryKey)
            }
            "x509-certificate" => serde_json::from_value(value).map(StixObject::X509Certificate),
            "marking-definition" => {
                serde_json::from_value(value).map(StixObject::MarkingDefinition)
            }
            "language-content" => serde_json::from_value(value).map(StixObject::LanguageContent),
            _ => {
                // Unknown type - store as custom
                serde_json::from_value(value).map(StixObject::Custom)
            }
        };

        result.map_err(serde::de::Error::custom)
    }
}

// Implement From for all object types
macro_rules! impl_from_stix_object {
    ($variant:ident, $type:ty) => {
        impl From<$type> for StixObject {
            fn from(obj: $type) -> Self {
                StixObject::$variant(obj)
            }
        }
    };
}

impl_from_stix_object!(AttackPattern, crate::objects::AttackPattern);
impl_from_stix_object!(Campaign, crate::objects::Campaign);
impl_from_stix_object!(CourseOfAction, crate::objects::CourseOfAction);
impl_from_stix_object!(Grouping, crate::objects::Grouping);
impl_from_stix_object!(Identity, crate::objects::Identity);
impl_from_stix_object!(Incident, crate::objects::Incident);
impl_from_stix_object!(Indicator, crate::objects::Indicator);
impl_from_stix_object!(Infrastructure, crate::objects::Infrastructure);
impl_from_stix_object!(IntrusionSet, crate::objects::IntrusionSet);
impl_from_stix_object!(Location, crate::objects::Location);
impl_from_stix_object!(Malware, crate::objects::Malware);
impl_from_stix_object!(MalwareAnalysis, crate::objects::MalwareAnalysis);
impl_from_stix_object!(Note, crate::objects::Note);
impl_from_stix_object!(ObservedData, crate::objects::ObservedData);
impl_from_stix_object!(Opinion, crate::objects::Opinion);
impl_from_stix_object!(Report, crate::objects::Report);
impl_from_stix_object!(ThreatActor, crate::objects::ThreatActor);
impl_from_stix_object!(Tool, crate::objects::Tool);
impl_from_stix_object!(Vulnerability, crate::objects::Vulnerability);
impl_from_stix_object!(Relationship, crate::relationship::Relationship);
impl_from_stix_object!(Sighting, crate::relationship::Sighting);
impl_from_stix_object!(Artifact, crate::observables::Artifact);
impl_from_stix_object!(AutonomousSystem, crate::observables::AutonomousSystem);
impl_from_stix_object!(Directory, crate::observables::Directory);
impl_from_stix_object!(DomainName, crate::observables::DomainName);
impl_from_stix_object!(EmailAddress, crate::observables::EmailAddress);
impl_from_stix_object!(EmailMessage, crate::observables::EmailMessage);
impl_from_stix_object!(File, crate::observables::File);
impl_from_stix_object!(IPv4Address, crate::observables::IPv4Address);
impl_from_stix_object!(IPv6Address, crate::observables::IPv6Address);
impl_from_stix_object!(MacAddress, crate::observables::MacAddress);
impl_from_stix_object!(Mutex, crate::observables::Mutex);
impl_from_stix_object!(NetworkTraffic, crate::observables::NetworkTraffic);
impl_from_stix_object!(Process, crate::observables::Process);
impl_from_stix_object!(Software, crate::observables::Software);
impl_from_stix_object!(Url, crate::observables::Url);
impl_from_stix_object!(UserAccount, crate::observables::UserAccount);
impl_from_stix_object!(WindowsRegistryKey, crate::observables::WindowsRegistryKey);
impl_from_stix_object!(X509Certificate, crate::observables::X509Certificate);
impl_from_stix_object!(MarkingDefinition, crate::markings::MarkingDefinition);
impl_from_stix_object!(LanguageContent, crate::objects::LanguageContent);

#[cfg(test)]
mod tests {

    #[test]
    fn test_stix_object_type_name() {
        // Tests will be added once object types are implemented
    }
}
