//! TAXII 1.x constants.
//!
//! This module provides both string constants for compatibility and type-safe enums
//! for compile-time validation of TAXII protocol values.

use std::fmt;
use std::str::FromStr;

// TAXII Version IDs (XML Bindings)
pub const VID_TAXII_XML_10: &str = "urn:taxii.mitre.org:message:xml:1.0";
pub const VID_TAXII_XML_11: &str = "urn:taxii.mitre.org:message:xml:1.1";

// TAXII Service IDs
pub const VID_TAXII_SERVICES_10: &str = "urn:taxii.mitre.org:services:1.0";
pub const VID_TAXII_SERVICES_11: &str = "urn:taxii.mitre.org:services:1.1";

// TAXII Protocol Bindings
pub const VID_TAXII_HTTP_10: &str = "urn:taxii.mitre.org:protocol:http:1.0";
pub const VID_TAXII_HTTPS_10: &str = "urn:taxii.mitre.org:protocol:https:1.0";

// Content Bindings
pub const CB_STIX_XML_10: &str = "urn:stix.mitre.org:xml:1.0";
pub const CB_STIX_XML_101: &str = "urn:stix.mitre.org:xml:1.0.1";
pub const CB_STIX_XML_11: &str = "urn:stix.mitre.org:xml:1.1";
pub const CB_STIX_XML_111: &str = "urn:stix.mitre.org:xml:1.1.1";
pub const CB_STIX_XML_12: &str = "urn:stix.mitre.org:xml:1.2";

// Collection Types
pub const CT_DATA_FEED: &str = "DATA_FEED";
pub const CT_DATA_SET: &str = "DATA_SET";

/// Type-safe collection type enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollectionType {
    DataFeed,
    DataSet,
}

impl CollectionType {
    /// Get the string representation.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DataFeed => CT_DATA_FEED,
            Self::DataSet => CT_DATA_SET,
        }
    }
}

impl FromStr for CollectionType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            CT_DATA_FEED => Ok(Self::DataFeed),
            CT_DATA_SET => Ok(Self::DataSet),
            _ => Err(()),
        }
    }
}

impl fmt::Display for CollectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// Response Types
pub const RT_FULL: &str = "FULL";
pub const RT_COUNT_ONLY: &str = "COUNT_ONLY";

/// Type-safe response type enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ResponseType {
    #[default]
    Full,
    CountOnly,
}

impl ResponseType {
    /// Get the string representation.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => RT_FULL,
            Self::CountOnly => RT_COUNT_ONLY,
        }
    }
}

impl FromStr for ResponseType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            RT_FULL => Ok(Self::Full),
            RT_COUNT_ONLY => Ok(Self::CountOnly),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ResponseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// Subscription Status
pub const SS_ACTIVE: &str = "ACTIVE";
pub const SS_PAUSED: &str = "PAUSED";
pub const SS_UNSUBSCRIBED: &str = "UNSUBSCRIBED";

/// Type-safe subscription status enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SubscriptionStatus {
    #[default]
    Active,
    Paused,
    Unsubscribed,
}

impl SubscriptionStatus {
    /// Get the string representation.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => SS_ACTIVE,
            Self::Paused => SS_PAUSED,
            Self::Unsubscribed => SS_UNSUBSCRIBED,
        }
    }
}

impl FromStr for SubscriptionStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            SS_ACTIVE => Ok(Self::Active),
            SS_PAUSED => Ok(Self::Paused),
            SS_UNSUBSCRIBED => Ok(Self::Unsubscribed),
            _ => Err(()),
        }
    }
}

impl fmt::Display for SubscriptionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// Service Types
pub const SVC_DISCOVERY: &str = "DISCOVERY";
pub const SVC_INBOX: &str = "INBOX";
pub const SVC_POLL: &str = "POLL";
pub const SVC_COLLECTION_MANAGEMENT: &str = "COLLECTION_MANAGEMENT";

/// Type-safe service type enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceType {
    Discovery,
    Inbox,
    Poll,
    CollectionManagement,
    /// TAXII 1.0 feed management (equivalent to collection management).
    FeedManagement,
}

impl ServiceType {
    /// Get the string representation.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Discovery => SVC_DISCOVERY,
            Self::Inbox => SVC_INBOX,
            Self::Poll => SVC_POLL,
            Self::CollectionManagement => SVC_COLLECTION_MANAGEMENT,
            Self::FeedManagement => SVC_FEED_MANAGEMENT,
        }
    }
}

impl FromStr for ServiceType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            SVC_DISCOVERY => Ok(Self::Discovery),
            SVC_INBOX => Ok(Self::Inbox),
            SVC_POLL => Ok(Self::Poll),
            SVC_COLLECTION_MANAGEMENT => Ok(Self::CollectionManagement),
            SVC_FEED_MANAGEMENT => Ok(Self::FeedManagement),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ServiceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// Status Types (TAXII 1.0)
pub const ST_BAD_MESSAGE: &str = "BAD_MESSAGE";
pub const ST_DENIED: &str = "DENIED";
pub const ST_NOT_FOUND: &str = "NOT_FOUND";
pub const ST_POLLING_UNSUPPORTED: &str = "POLLING_UNSUPPORTED";
pub const ST_UNAUTHORIZED: &str = "UNAUTHORIZED";
pub const ST_UNSUPPORTED_MESSAGE_BINDING: &str = "UNSUPPORTED_MESSAGE";
pub const ST_UNSUPPORTED_CONTENT_BINDING: &str = "UNSUPPORTED_CONTENT";
pub const ST_UNSUPPORTED_PROTOCOL: &str = "UNSUPPORTED_PROTOCOL";

// Status Types (TAXII 1.1)
pub const ST_SUCCESS: &str = "SUCCESS";
pub const ST_FAILURE: &str = "FAILURE";
pub const ST_PENDING: &str = "PENDING";
pub const ST_RETRY: &str = "RETRY";
pub const ST_UNAVAILABLE: &str = "UNAVAILABLE";
pub const ST_ASYNCHRONOUS_POLL_ERROR: &str = "ASYNCHRONOUS_POLL_ERROR";
pub const ST_DESTINATION_COLLECTION_ERROR: &str = "DESTINATION_COLLECTION_ERROR";
pub const ST_INVALID_RESPONSE_PART: &str = "INVALID_RESPONSE_PART";
pub const ST_NETWORK_ERROR: &str = "NETWORK_ERROR";
pub const ST_UNSUPPORTED_QUERY: &str = "UNSUPPORTED_QUERY";

/// Type-safe status type enum (covers both TAXII 1.0 and 1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusType {
    // Common statuses
    Success,
    Failure,
    // TAXII 1.0 statuses
    BadMessage,
    Denied,
    NotFound,
    PollingUnsupported,
    Unauthorized,
    UnsupportedMessageBinding,
    UnsupportedContentBinding,
    UnsupportedProtocol,
    // TAXII 1.1 additional statuses
    Pending,
    Retry,
    Unavailable,
    AsynchronousPollError,
    DestinationCollectionError,
    InvalidResponsePart,
    NetworkError,
    UnsupportedQuery,
}

impl StatusType {
    /// Get the string representation.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => ST_SUCCESS,
            Self::Failure => ST_FAILURE,
            Self::BadMessage => ST_BAD_MESSAGE,
            Self::Denied => ST_DENIED,
            Self::NotFound => ST_NOT_FOUND,
            Self::PollingUnsupported => ST_POLLING_UNSUPPORTED,
            Self::Unauthorized => ST_UNAUTHORIZED,
            Self::UnsupportedMessageBinding => ST_UNSUPPORTED_MESSAGE_BINDING,
            Self::UnsupportedContentBinding => ST_UNSUPPORTED_CONTENT_BINDING,
            Self::UnsupportedProtocol => ST_UNSUPPORTED_PROTOCOL,
            Self::Pending => ST_PENDING,
            Self::Retry => ST_RETRY,
            Self::Unavailable => ST_UNAVAILABLE,
            Self::AsynchronousPollError => ST_ASYNCHRONOUS_POLL_ERROR,
            Self::DestinationCollectionError => ST_DESTINATION_COLLECTION_ERROR,
            Self::InvalidResponsePart => ST_INVALID_RESPONSE_PART,
            Self::NetworkError => ST_NETWORK_ERROR,
            Self::UnsupportedQuery => ST_UNSUPPORTED_QUERY,
        }
    }

    /// Check if this is an error status.
    #[inline]
    pub const fn is_error(self) -> bool {
        !matches!(self, Self::Success | Self::Pending)
    }
}

impl FromStr for StatusType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ST_SUCCESS => Ok(Self::Success),
            ST_FAILURE => Ok(Self::Failure),
            ST_BAD_MESSAGE => Ok(Self::BadMessage),
            ST_DENIED => Ok(Self::Denied),
            ST_NOT_FOUND => Ok(Self::NotFound),
            ST_POLLING_UNSUPPORTED => Ok(Self::PollingUnsupported),
            ST_UNAUTHORIZED => Ok(Self::Unauthorized),
            ST_UNSUPPORTED_MESSAGE_BINDING => Ok(Self::UnsupportedMessageBinding),
            ST_UNSUPPORTED_CONTENT_BINDING => Ok(Self::UnsupportedContentBinding),
            ST_UNSUPPORTED_PROTOCOL => Ok(Self::UnsupportedProtocol),
            ST_PENDING => Ok(Self::Pending),
            ST_RETRY => Ok(Self::Retry),
            ST_UNAVAILABLE => Ok(Self::Unavailable),
            ST_ASYNCHRONOUS_POLL_ERROR => Ok(Self::AsynchronousPollError),
            ST_DESTINATION_COLLECTION_ERROR => Ok(Self::DestinationCollectionError),
            ST_INVALID_RESPONSE_PART => Ok(Self::InvalidResponsePart),
            ST_NETWORK_ERROR => Ok(Self::NetworkError),
            ST_UNSUPPORTED_QUERY => Ok(Self::UnsupportedQuery),
            _ => Err(()),
        }
    }
}

impl fmt::Display for StatusType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// Action Types (for Collection Management)
pub const ACT_SUBSCRIBE: &str = "SUBSCRIBE";
pub const ACT_UNSUBSCRIBE: &str = "UNSUBSCRIBE";
pub const ACT_PAUSE: &str = "PAUSE";
pub const ACT_RESUME: &str = "RESUME";
pub const ACT_STATUS: &str = "STATUS";

/// Type-safe subscription action enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubscriptionAction {
    Subscribe,
    Unsubscribe,
    Pause,
    Resume,
    Status,
}

impl SubscriptionAction {
    /// Get the string representation.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Subscribe => ACT_SUBSCRIBE,
            Self::Unsubscribe => ACT_UNSUBSCRIBE,
            Self::Pause => ACT_PAUSE,
            Self::Resume => ACT_RESUME,
            Self::Status => ACT_STATUS,
        }
    }

    /// Check if this action is valid for TAXII 1.0.
    #[inline]
    pub const fn is_valid_for_10(self) -> bool {
        matches!(self, Self::Subscribe | Self::Unsubscribe | Self::Status)
    }

    /// Check if this action is valid for TAXII 1.1.
    #[inline]
    pub const fn is_valid_for_11(self) -> bool {
        true // All actions are valid for 1.1
    }
}

impl FromStr for SubscriptionAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ACT_SUBSCRIBE => Ok(Self::Subscribe),
            ACT_UNSUBSCRIBE => Ok(Self::Unsubscribe),
            ACT_PAUSE => Ok(Self::Pause),
            ACT_RESUME => Ok(Self::Resume),
            ACT_STATUS => Ok(Self::Status),
            _ => Err(()),
        }
    }
}

impl fmt::Display for SubscriptionAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// Message types (for reference)
pub const MSG_DISCOVERY_REQUEST: &str = "Discovery_Request";
pub const MSG_DISCOVERY_RESPONSE: &str = "Discovery_Response";
pub const MSG_POLL_REQUEST: &str = "Poll_Request";
pub const MSG_POLL_RESPONSE: &str = "Poll_Response";
pub const MSG_POLL_FULFILLMENT_REQUEST: &str = "Poll_Fulfillment";
pub const MSG_INBOX_MESSAGE: &str = "Inbox_Message";
pub const MSG_STATUS_MESSAGE: &str = "Status_Message";

// TAXII 1.1 Message Types
pub const MSG_COLLECTION_INFORMATION_REQUEST: &str = "Collection_Information_Request";
pub const MSG_COLLECTION_INFORMATION_RESPONSE: &str = "Collection_Information_Response";
pub const MSG_MANAGE_COLLECTION_SUBSCRIPTION_REQUEST: &str = "Subscription_Management_Request";
pub const MSG_MANAGE_COLLECTION_SUBSCRIPTION_RESPONSE: &str = "Subscription_Management_Response";

// TAXII 1.0 Message Types
pub const MSG_FEED_INFORMATION_REQUEST: &str = "Feed_Information_Request";
pub const MSG_FEED_INFORMATION_RESPONSE: &str = "Feed_Information_Response";
pub const MSG_MANAGE_FEED_SUBSCRIPTION_REQUEST: &str = "Subscription_Management_Request";
pub const MSG_MANAGE_FEED_SUBSCRIPTION_RESPONSE: &str = "Subscription_Management_Response";

// Service Types (TAXII 1.0)
pub const SVC_FEED_MANAGEMENT: &str = "FEED_MANAGEMENT";

// Status Detail Keys (for ST_PENDING and other status messages)
/// Estimated wait time in seconds for async poll results.
pub const SD_ESTIMATED_WAIT: &str = "ESTIMATED_WAIT";
/// Result ID for poll fulfillment request.
pub const SD_RESULT_ID: &str = "RESULT_ID";
/// Whether the server will push results when ready.
pub const SD_WILL_PUSH: &str = "WILL_PUSH";
/// Supported content bindings.
pub const SD_SUPPORTED_CONTENT: &str = "SUPPORTED_CONTENT";
/// Acceptable destination collection names.
pub const SD_ACCEPTABLE_DESTINATION: &str = "ACCEPTABLE_DESTINATION";
/// Maximum part number for poll fulfillment.
pub const SD_MAX_PART_NUMBER: &str = "MAX_PART_NUMBER";
/// Item field for status details.
pub const SD_ITEM: &str = "ITEM";
/// Supported message bindings.
pub const SD_SUPPORTED_BINDING: &str = "SUPPORTED_BINDING";
/// Supported protocol bindings.
pub const SD_SUPPORTED_PROTOCOL: &str = "SUPPORTED_PROTOCOL";
/// Supported query formats.
pub const SD_SUPPORTED_QUERY: &str = "SUPPORTED_QUERY";

// Content Bindings (additional)
/// CAP 1.1 content binding.
pub const CB_CAP_11: &str = "urn:oasis:names:tc:emergency:cap:1.1";
/// XML Encryption content binding.
pub const CB_XENC_122002: &str = "http://www.w3.org/2001/04/xmlenc#";
/// S/MIME content binding.
pub const CB_SMIME: &str = "application/x-pkcs7-mime";

// Action type arrays for validation
/// Valid action types for TAXII 1.0 subscription management.
pub const ACT_TYPES_10: &[&str] = &[ACT_SUBSCRIBE, ACT_UNSUBSCRIBE, ACT_STATUS];
/// Valid action types for TAXII 1.1 subscription management.
pub const ACT_TYPES_11: &[&str] = &[
    ACT_SUBSCRIBE,
    ACT_UNSUBSCRIBE,
    ACT_PAUSE,
    ACT_RESUME,
    ACT_STATUS,
];
