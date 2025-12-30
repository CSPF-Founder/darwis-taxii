//! Common types and utilities for TAXII 1.x messages.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// TAXII 1.0 namespace
pub const NS_TAXII_10: &str = "http://taxii.mitre.org/messages/taxii_xml_binding-1";

/// TAXII 1.1 namespace
pub const NS_TAXII_11: &str = "http://taxii.mitre.org/messages/taxii_xml_binding-1.1";

/// Generate a message ID.
pub fn generate_message_id() -> String {
    Uuid::new_v4().to_string()
}

/// Extended header for TAXII messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtendedHeader {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// Content binding for TAXII 1.x.
///
/// In TAXII 1.0, content bindings are simple strings.
/// In TAXII 1.1, they can have subtypes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentBinding {
    /// The binding ID (e.g., "urn:stix.mitre.org:xml:1.1")
    #[serde(rename = "@binding_id")]
    pub binding_id: String,

    /// Optional subtype IDs (TAXII 1.1 only)
    #[serde(rename = "Subtype", default)]
    pub subtype_ids: Vec<Subtype>,
}

impl ContentBinding {
    /// Create a new content binding with just a binding ID.
    pub fn new(binding_id: impl Into<String>) -> Self {
        Self {
            binding_id: binding_id.into(),
            subtype_ids: Vec::new(),
        }
    }

    /// Create a content binding with subtypes.
    pub fn with_subtypes(binding_id: impl Into<String>, subtypes: Vec<String>) -> Self {
        Self {
            binding_id: binding_id.into(),
            subtype_ids: subtypes
                .into_iter()
                .map(|s| Subtype { subtype_id: s })
                .collect(),
        }
    }
}

/// Subtype for content binding (TAXII 1.1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subtype {
    #[serde(rename = "@subtype_id")]
    pub subtype_id: String,
}

/// Timestamp label (datetime).
pub type TimestampLabel = chrono::DateTime<chrono::Utc>;

/// Record count with partial flag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecordCount {
    #[serde(rename = "@partial_count")]
    pub partial_count: bool,
    #[serde(rename = "$text")]
    pub record_count: i64,
}

/// Subscription information in inbox messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscriptionInformation {
    /// Collection name (TAXII 1.1) or Feed name (TAXII 1.0)
    #[serde(rename = "Collection_Name", alias = "Feed_Name")]
    pub collection_name: String,

    /// Subscription ID
    #[serde(rename = "Subscription_ID")]
    pub subscription_id: String,

    /// Exclusive begin timestamp label
    #[serde(
        rename = "Exclusive_Begin_Timestamp_Label",
        skip_serializing_if = "Option::is_none"
    )]
    pub exclusive_begin_timestamp_label: Option<String>,

    /// Inclusive end timestamp label
    #[serde(
        rename = "Inclusive_End_Timestamp_Label",
        skip_serializing_if = "Option::is_none"
    )]
    pub inclusive_end_timestamp_label: Option<String>,
}

/// Push parameters for subscriptions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PushParameters {
    /// Protocol binding
    #[serde(rename = "Protocol_Binding")]
    pub protocol_binding: String,

    /// Address to push to
    #[serde(rename = "Address")]
    pub address: String,

    /// Message binding
    #[serde(rename = "Message_Binding")]
    pub message_binding: String,
}

/// Subscription parameters (TAXII 1.1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscriptionParameters {
    /// Response type (FULL or COUNT_ONLY)
    #[serde(rename = "Response_Type", skip_serializing_if = "Option::is_none")]
    pub response_type: Option<String>,

    /// Content bindings
    #[serde(rename = "Content_Binding", default)]
    pub content_bindings: Vec<ContentBinding>,
}

/// Status detail for status messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusDetail {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}
