//! TAXII 1.x entities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::Account;

/// TAXII 1.x collection types.
///
/// From libtaxii.constants: CT_DATA_FEED, CT_DATA_SET
pub mod collection_type {
    pub const DATA_FEED: &str = "DATA_FEED";
    pub const DATA_SET: &str = "DATA_SET";
}

/// TAXII 1.x response types.
///
/// From libtaxii.constants: RT_FULL, RT_COUNT_ONLY
pub mod response_type {
    pub const FULL: &str = "FULL";
    pub const COUNT_ONLY: &str = "COUNT_ONLY";
}

/// TAXII 1.x subscription status.
///
/// From libtaxii.constants: SS_ACTIVE, SS_PAUSED, SS_UNSUBSCRIBED
pub mod subscription_status {
    pub const ACTIVE: &str = "ACTIVE";
    pub const PAUSED: &str = "PAUSED";
    pub const UNSUBSCRIBED: &str = "UNSUBSCRIBED";
}

/// TAXII Service entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntity {
    /// Service ID.
    pub id: Option<String>,

    /// Service type (inbox, poll, discovery, collection_management).
    #[serde(rename = "type")]
    pub service_type: String,

    /// Service-specific properties.
    pub properties: serde_json::Value,
}

/// Content Binding entity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentBindingEntity {
    /// Content binding ID.
    pub binding: String,

    /// List of subtype IDs.
    #[serde(default)]
    pub subtypes: Vec<String>,
}

impl ContentBindingEntity {
    /// Create new content binding.
    pub fn new(binding: impl Into<String>) -> Self {
        Self {
            binding: binding.into(),
            subtypes: Vec::new(),
        }
    }

    /// Create new content binding with subtypes.
    pub fn with_subtypes(binding: impl Into<String>, subtypes: Vec<String>) -> Self {
        Self {
            binding: binding.into(),
            subtypes,
        }
    }

    /// Serialize content bindings to JSON string for database storage.
    ///
    /// Format: `[["binding_id", ["subtype1", "subtype2"]], ...]`
    pub fn serialize_many(bindings: &[Self]) -> String {
        let data: Vec<(&str, &[String])> = bindings
            .iter()
            .map(|b| (b.binding.as_str(), b.subtypes.as_slice()))
            .collect();
        serde_json::to_string(&data).unwrap_or_else(|_| "[]".to_string())
    }

    /// Deserialize content bindings from JSON string.
    ///
    /// Expects format: `[["binding_id", ["subtype1", "subtype2"]], ...]`
    pub fn deserialize_many(data: &str) -> Vec<Self> {
        let raw: Vec<(String, Vec<String>)> = serde_json::from_str(data).unwrap_or_default();
        raw.into_iter()
            .map(|(binding, subtypes)| Self::with_subtypes(binding, subtypes))
            .collect()
    }
}

/// Collection entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionEntity {
    /// Collection ID.
    pub id: Option<i32>,

    /// Collection name.
    pub name: String,

    /// Whether collection is available.
    pub available: bool,

    /// Collection volume (number of content blocks).
    pub volume: Option<i32>,

    /// Collection description.
    pub description: Option<String>,

    /// Whether collection accepts all content types.
    pub accept_all_content: bool,

    /// Collection type (DATA_FEED or DATA_SET).
    #[serde(rename = "type")]
    pub collection_type: String,

    /// List of supported content bindings.
    pub supported_content: Vec<ContentBindingEntity>,
}

impl CollectionEntity {
    /// Check if content binding is supported.
    pub fn is_content_supported(&self, content_binding: &ContentBindingEntity) -> bool {
        if self.accept_all_content {
            return true;
        }

        self.supported_content.iter().any(|supported| {
            if supported.binding != content_binding.binding {
                return false;
            }
            // If either side has no subtypes specified, any subtype matches
            if supported.subtypes.is_empty() || content_binding.subtypes.is_empty() {
                return true;
            }
            // Check if any requested subtype is in the supported set
            let supported_set: HashSet<_> = supported.subtypes.iter().collect();
            content_binding
                .subtypes
                .iter()
                .any(|subtype| supported_set.contains(subtype))
        })
    }

    /// Get matching bindings between requested and supported.
    pub fn get_matching_bindings(
        &self,
        requested_bindings: &[ContentBindingEntity],
    ) -> Vec<ContentBindingEntity> {
        if self.accept_all_content {
            return requested_bindings.to_vec();
        }

        if self.supported_content.is_empty() {
            return requested_bindings.to_vec();
        }

        if requested_bindings.is_empty() {
            return self.supported_content.clone();
        }

        let mut overlap = Vec::new();

        for requested in requested_bindings {
            for supported in &self.supported_content {
                if requested.binding != supported.binding {
                    continue;
                }

                if supported.subtypes.is_empty() {
                    overlap.push(requested.clone());
                    continue;
                }

                if requested.subtypes.is_empty() {
                    overlap.push(supported.clone());
                    continue;
                }

                let supported_set: HashSet<_> = supported.subtypes.iter().collect();
                let subtypes_overlap: Vec<String> = requested
                    .subtypes
                    .iter()
                    .filter(|s| supported_set.contains(s))
                    .cloned()
                    .collect();

                overlap.push(ContentBindingEntity {
                    binding: requested.binding.clone(),
                    subtypes: subtypes_overlap,
                });
            }
        }

        overlap
    }

    /// Check if account can read from this collection.
    ///
    /// Uses collection NAME as the permission key (TAXII 1.x style).
    pub fn can_read(&self, account: &Account) -> bool {
        account.is_admin
            || account
                .permissions
                .get(&self.name)
                .map(|p| p.can_read())
                .unwrap_or(false)
    }

    /// Check if account can modify this collection.
    ///
    /// Uses collection NAME as the permission key (TAXII 1.x style).
    pub fn can_modify(&self, account: &Account) -> bool {
        account.is_admin
            || account
                .permissions
                .get(&self.name)
                .map(|p| p.can_write())
                .unwrap_or(false)
    }
}

/// Content Block entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlockEntity {
    /// Content block ID.
    pub id: Option<i32>,

    /// Content payload (the actual STIX/etc content).
    pub content: Vec<u8>,

    /// Timestamp label.
    pub timestamp_label: DateTime<Utc>,

    /// Content binding.
    pub content_binding: Option<ContentBindingEntity>,

    /// Message attached to the content block.
    pub message: Option<String>,

    /// Internal ID of the inbox message.
    pub inbox_message_id: Option<i32>,
}

/// Inbox Message entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxMessageEntity {
    /// Internal ID.
    pub id: Option<i32>,

    /// TAXII message ID.
    pub message_id: String,

    /// XML serialized original TAXII message.
    pub original_message: Vec<u8>,

    /// How many content blocks this message contains.
    pub content_block_count: i32,

    /// ID of the Inbox Service that received the message.
    pub service_id: String,

    /// Destination collections.
    pub destination_collections: Vec<String>,

    /// ID of the Result Set.
    pub result_id: Option<String>,

    /// How many items left in the Result Set.
    pub record_count: Option<i32>,

    /// If the record count is partial.
    pub partial_count: bool,

    /// Subscription collection name.
    pub subscription_collection_name: Option<String>,

    /// Subscription ID.
    pub subscription_id: Option<String>,

    /// Exclusive begin timestamp label.
    pub exclusive_begin_timestamp_label: Option<DateTime<Utc>>,

    /// Inclusive end timestamp label.
    pub inclusive_end_timestamp_label: Option<DateTime<Utc>>,
}

/// Result Set entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSetEntity {
    /// Result set ID.
    pub id: String,

    /// Collection ID.
    pub collection_id: i32,

    /// Content bindings filter.
    pub content_bindings: Vec<ContentBindingEntity>,

    /// Timeframe as (begin, end).
    pub timeframe: (Option<DateTime<Utc>>, Option<DateTime<Utc>>),
}

/// Subscription Parameters entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionParameters {
    /// Response type (FULL or COUNT_ONLY).
    pub response_type: String,

    /// Content bindings filter.
    #[serde(default)]
    pub content_bindings: Vec<ContentBindingEntity>,
}

impl Default for SubscriptionParameters {
    fn default() -> Self {
        Self {
            response_type: response_type::FULL.to_string(),
            content_bindings: Vec::new(),
        }
    }
}

/// Poll Request Parameters entity.
pub type PollRequestParametersEntity = SubscriptionParameters;

/// Subscription entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionEntity {
    /// Service ID.
    pub service_id: String,

    /// Collection ID.
    pub collection_id: i32,

    /// Subscription ID.
    pub subscription_id: Option<String>,

    /// Poll request parameters.
    pub params: Option<PollRequestParametersEntity>,

    /// Subscription status (ACTIVE, PAUSED, UNSUBSCRIBED).
    pub status: String,
}
