//! TAXII 1.0 message types.

use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};

use super::common::NS_TAXII_10;
use crate::constants::{
    MSG_DISCOVERY_REQUEST, MSG_DISCOVERY_RESPONSE, MSG_FEED_INFORMATION_REQUEST,
    MSG_FEED_INFORMATION_RESPONSE, MSG_INBOX_MESSAGE, MSG_MANAGE_FEED_SUBSCRIPTION_REQUEST,
    MSG_MANAGE_FEED_SUBSCRIPTION_RESPONSE, MSG_POLL_REQUEST, MSG_POLL_RESPONSE, MSG_STATUS_MESSAGE,
    ST_FAILURE, ST_SUCCESS,
};
use crate::error::{Taxii1xError, Taxii1xResult};

// Re-export common types
pub use super::common::{ExtendedHeader, RecordCount, StatusDetail, SubscriptionInformation};

/// Wrapper enum for all TAXII 1.0 message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Taxii10Message {
    #[serde(rename = "Discovery_Request")]
    DiscoveryRequest(DiscoveryRequest),
    #[serde(rename = "Discovery_Response")]
    DiscoveryResponse(DiscoveryResponse),
    #[serde(rename = "Feed_Information_Request")]
    FeedInformationRequest(FeedInformationRequest),
    #[serde(rename = "Feed_Information_Response")]
    FeedInformationResponse(FeedInformationResponse),
    #[serde(rename = "Poll_Request")]
    PollRequest(PollRequest),
    #[serde(rename = "Poll_Response")]
    PollResponse(PollResponse),
    #[serde(rename = "Inbox_Message")]
    InboxMessage(InboxMessage),
    #[serde(rename = "Status_Message")]
    StatusMessage(StatusMessage),
    #[serde(rename = "Subscription_Management_Request")]
    ManageFeedSubscriptionRequest(ManageFeedSubscriptionRequest),
    #[serde(rename = "Subscription_Management_Response")]
    ManageFeedSubscriptionResponse(ManageFeedSubscriptionResponse),
}

impl Taxii10Message {
    /// Get message ID.
    pub fn message_id(&self) -> &str {
        match self {
            Self::DiscoveryRequest(m) => &m.message_id,
            Self::DiscoveryResponse(m) => &m.message_id,
            Self::FeedInformationRequest(m) => &m.message_id,
            Self::FeedInformationResponse(m) => &m.message_id,
            Self::PollRequest(m) => &m.message_id,
            Self::PollResponse(m) => &m.message_id,
            Self::InboxMessage(m) => &m.message_id,
            Self::StatusMessage(m) => &m.message_id,
            Self::ManageFeedSubscriptionRequest(m) => &m.message_id,
            Self::ManageFeedSubscriptionResponse(m) => &m.message_id,
        }
    }

    /// Get message type.
    pub fn message_type(&self) -> &str {
        match self {
            Self::DiscoveryRequest(_) => MSG_DISCOVERY_REQUEST,
            Self::DiscoveryResponse(_) => MSG_DISCOVERY_RESPONSE,
            Self::FeedInformationRequest(_) => MSG_FEED_INFORMATION_REQUEST,
            Self::FeedInformationResponse(_) => MSG_FEED_INFORMATION_RESPONSE,
            Self::PollRequest(_) => MSG_POLL_REQUEST,
            Self::PollResponse(_) => MSG_POLL_RESPONSE,
            Self::InboxMessage(_) => MSG_INBOX_MESSAGE,
            Self::StatusMessage(_) => MSG_STATUS_MESSAGE,
            Self::ManageFeedSubscriptionRequest(_) => MSG_MANAGE_FEED_SUBSCRIPTION_REQUEST,
            Self::ManageFeedSubscriptionResponse(_) => MSG_MANAGE_FEED_SUBSCRIPTION_RESPONSE,
        }
    }

    /// Serialize to XML.
    pub fn to_xml(&self) -> Taxii1xResult<String> {
        to_string(self).map_err(Taxii1xError::xml_serialize)
    }
}

// ============================================================================
// Discovery Messages
// ============================================================================

/// TAXII 1.0 Discovery Request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Discovery_Request")]
pub struct DiscoveryRequest {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_10")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,
}

impl DiscoveryRequest {
    /// Create a new discovery request.
    pub fn new(message_id: impl Into<String>) -> Self {
        Self {
            xmlns: NS_TAXII_10.to_string(),
            message_id: message_id.into(),
            extended_headers: None,
        }
    }
}

/// TAXII 1.0 Discovery Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Discovery_Response")]
pub struct DiscoveryResponse {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_10")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@in_response_to", skip_serializing_if = "Option::is_none")]
    pub in_response_to: Option<String>,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(rename = "Service_Instance", default)]
    pub service_instances: Vec<ServiceInstance>,
}

impl DiscoveryResponse {
    /// Create a new discovery response.
    pub fn new(message_id: impl Into<String>, in_response_to: impl Into<String>) -> Self {
        Self {
            xmlns: NS_TAXII_10.to_string(),
            message_id: message_id.into(),
            in_response_to: Some(in_response_to.into()),
            extended_headers: None,
            service_instances: Vec::new(),
        }
    }
}

// ============================================================================
// Feed Information Messages
// ============================================================================

/// TAXII 1.0 Feed Information Request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Feed_Information_Request")]
pub struct FeedInformationRequest {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_10")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,
}

impl FeedInformationRequest {
    /// Create a new feed information request.
    pub fn new(message_id: impl Into<String>) -> Self {
        Self {
            xmlns: NS_TAXII_10.to_string(),
            message_id: message_id.into(),
            extended_headers: None,
        }
    }
}

/// TAXII 1.0 Feed Information Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Feed_Information_Response")]
pub struct FeedInformationResponse {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_10")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@in_response_to", skip_serializing_if = "Option::is_none")]
    pub in_response_to: Option<String>,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(rename = "Feed", default)]
    pub feeds: Vec<FeedInformation>,
}

impl FeedInformationResponse {
    /// Create a new feed information response.
    pub fn new(message_id: impl Into<String>, in_response_to: impl Into<String>) -> Self {
        Self {
            xmlns: NS_TAXII_10.to_string(),
            message_id: message_id.into(),
            in_response_to: Some(in_response_to.into()),
            extended_headers: None,
            feeds: Vec::new(),
        }
    }
}

// ============================================================================
// Poll Messages
// ============================================================================

/// TAXII 1.0 Poll Request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Poll_Request")]
pub struct PollRequest {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_10")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "Feed_Name")]
    pub feed_name: String,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(rename = "Subscription_ID", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,

    #[serde(
        rename = "Exclusive_Begin_Timestamp_Label",
        skip_serializing_if = "Option::is_none"
    )]
    pub exclusive_begin_timestamp_label: Option<String>,

    #[serde(
        rename = "Inclusive_End_Timestamp_Label",
        skip_serializing_if = "Option::is_none"
    )]
    pub inclusive_end_timestamp_label: Option<String>,

    #[serde(rename = "Content_Binding", default)]
    pub content_bindings: Vec<String>,
}

/// TAXII 1.0 Poll Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Poll_Response")]
pub struct PollResponse {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_10")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@in_response_to", skip_serializing_if = "Option::is_none")]
    pub in_response_to: Option<String>,

    #[serde(rename = "Feed_Name")]
    pub feed_name: String,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(
        rename = "Inclusive_Begin_Timestamp_Label",
        skip_serializing_if = "Option::is_none"
    )]
    pub inclusive_begin_timestamp_label: Option<String>,

    #[serde(
        rename = "Inclusive_End_Timestamp_Label",
        skip_serializing_if = "Option::is_none"
    )]
    pub inclusive_end_timestamp_label: Option<String>,

    #[serde(rename = "Subscription_ID", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,

    #[serde(rename = "Message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(rename = "Content_Block", default)]
    pub content_blocks: Vec<ContentBlock>,
}

impl PollResponse {
    /// Create a new poll response.
    pub fn new(
        message_id: impl Into<String>,
        in_response_to: impl Into<String>,
        feed_name: impl Into<String>,
    ) -> Self {
        Self {
            xmlns: NS_TAXII_10.to_string(),
            message_id: message_id.into(),
            in_response_to: Some(in_response_to.into()),
            feed_name: feed_name.into(),
            extended_headers: None,
            inclusive_begin_timestamp_label: None,
            inclusive_end_timestamp_label: None,
            subscription_id: None,
            message: None,
            content_blocks: Vec::new(),
        }
    }
}

// ============================================================================
// Inbox Messages
// ============================================================================

/// TAXII 1.0 Inbox Message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Inbox_Message")]
pub struct InboxMessage {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_10")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(rename = "Message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(
        rename = "Source_Subscription",
        skip_serializing_if = "Option::is_none"
    )]
    pub subscription_information: Option<SubscriptionInformation10>,

    #[serde(rename = "Content_Block", default)]
    pub content_blocks: Vec<ContentBlock>,
}

/// Subscription information for TAXII 1.0 inbox messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionInformation10 {
    #[serde(rename = "Feed_Name")]
    pub feed_name: String,

    #[serde(rename = "Subscription_ID")]
    pub subscription_id: String,

    #[serde(
        rename = "Inclusive_Begin_Timestamp_Label",
        skip_serializing_if = "Option::is_none"
    )]
    pub inclusive_begin_timestamp_label: Option<String>,

    #[serde(
        rename = "Inclusive_End_Timestamp_Label",
        skip_serializing_if = "Option::is_none"
    )]
    pub inclusive_end_timestamp_label: Option<String>,
}

// ============================================================================
// Status Messages
// ============================================================================

/// TAXII 1.0 Status Message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Status_Message")]
pub struct StatusMessage {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_10")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@in_response_to", skip_serializing_if = "Option::is_none")]
    pub in_response_to: Option<String>,

    #[serde(rename = "@status_type")]
    pub status_type: String,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(rename = "Status_Detail", skip_serializing_if = "Option::is_none")]
    pub status_detail: Option<String>,

    #[serde(rename = "Message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl StatusMessage {
    /// Create a new status message.
    pub fn new(
        message_id: impl Into<String>,
        in_response_to: Option<String>,
        status_type: impl Into<String>,
    ) -> Self {
        Self {
            xmlns: NS_TAXII_10.to_string(),
            message_id: message_id.into(),
            in_response_to,
            status_type: status_type.into(),
            extended_headers: None,
            status_detail: None,
            message: None,
        }
    }

    /// Create a success status message.
    pub fn success(message_id: impl Into<String>, in_response_to: impl Into<String>) -> Self {
        Self::new(message_id, Some(in_response_to.into()), ST_SUCCESS)
    }

    /// Create a failure status message.
    pub fn failure(
        message_id: impl Into<String>,
        in_response_to: Option<String>,
        message: Option<String>,
    ) -> Self {
        let mut status = Self::new(message_id, in_response_to, ST_FAILURE);
        status.message = message;
        status
    }

    /// Set status detail.
    pub fn with_status_detail(mut self, status_detail: impl Into<String>) -> Self {
        self.status_detail = Some(status_detail.into());
        self
    }
}

// ============================================================================
// Subscription Messages
// ============================================================================

/// TAXII 1.0 Manage Feed Subscription Request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Subscription_Management_Request")]
pub struct ManageFeedSubscriptionRequest {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_10")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@action")]
    pub action: String,

    #[serde(rename = "Feed_Name")]
    pub feed_name: String,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(rename = "Subscription_ID", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,

    #[serde(
        rename = "Delivery_Parameters",
        skip_serializing_if = "Option::is_none"
    )]
    pub delivery_parameters: Option<DeliveryParameters>,
}

/// Delivery parameters for TAXII 1.0 subscriptions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryParameters {
    #[serde(rename = "Inbox_Protocol")]
    pub inbox_protocol: String,

    #[serde(rename = "Inbox_Address")]
    pub inbox_address: String,

    #[serde(rename = "Delivery_Message_Binding")]
    pub delivery_message_binding: String,

    #[serde(rename = "Content_Binding", default)]
    pub content_bindings: Vec<String>,
}

/// TAXII 1.0 Manage Feed Subscription Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Subscription_Management_Response")]
pub struct ManageFeedSubscriptionResponse {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_10")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@in_response_to", skip_serializing_if = "Option::is_none")]
    pub in_response_to: Option<String>,

    #[serde(rename = "Feed_Name")]
    pub feed_name: String,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(rename = "Message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(rename = "Subscription", default)]
    pub subscription_instances: Vec<SubscriptionInstance>,
}

impl ManageFeedSubscriptionResponse {
    /// Create a new subscription response.
    pub fn new(
        message_id: impl Into<String>,
        in_response_to: impl Into<String>,
        feed_name: impl Into<String>,
    ) -> Self {
        Self {
            xmlns: NS_TAXII_10.to_string(),
            message_id: message_id.into(),
            in_response_to: Some(in_response_to.into()),
            feed_name: feed_name.into(),
            extended_headers: None,
            message: None,
            subscription_instances: Vec::new(),
        }
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// Extended headers container.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtendedHeaders {
    #[serde(rename = "Extended_Header", default)]
    pub headers: Vec<ExtendedHeader>,
}

/// Service instance for discovery responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    #[serde(rename = "@service_type")]
    pub service_type: String,

    #[serde(rename = "@service_version")]
    pub services_version: String,

    #[serde(rename = "@available", skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,

    #[serde(rename = "Protocol_Binding")]
    pub protocol_binding: String,

    #[serde(rename = "Address")]
    pub service_address: String,

    #[serde(rename = "Message_Binding", default)]
    pub message_bindings: Vec<String>,

    #[serde(rename = "Content_Binding", default)]
    pub inbox_service_accepted_content: Vec<String>,

    #[serde(rename = "Message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ServiceInstance {
    /// Create a new service instance.
    pub fn new(
        service_type: impl Into<String>,
        services_version: impl Into<String>,
        protocol_binding: impl Into<String>,
        service_address: impl Into<String>,
        message_bindings: Vec<String>,
    ) -> Self {
        Self {
            service_type: service_type.into(),
            services_version: services_version.into(),
            protocol_binding: protocol_binding.into(),
            service_address: service_address.into(),
            message_bindings,
            available: None,
            inbox_service_accepted_content: Vec::new(),
            message: None,
        }
    }
}

/// Feed information for feed information responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedInformation {
    #[serde(rename = "@feed_name")]
    pub feed_name: String,

    #[serde(rename = "@available", skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,

    #[serde(rename = "Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "Content_Binding", default)]
    pub supported_contents: Vec<String>,

    #[serde(rename = "Polling_Service", default)]
    pub polling_service_instances: Vec<PollingServiceInstance>,

    #[serde(rename = "Subscription_Service", default)]
    pub subscription_methods: Vec<SubscriptionMethod>,
    // Push_Method not implemented yet
}

/// Polling service instance for feed information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollingServiceInstance {
    #[serde(rename = "Protocol_Binding")]
    pub poll_protocol: String,

    #[serde(rename = "Address")]
    pub poll_address: String,

    #[serde(rename = "Message_Binding", default)]
    pub poll_message_bindings: Vec<String>,
}

/// Subscription method for feed information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionMethod {
    #[serde(rename = "Protocol_Binding")]
    pub subscription_protocol: String,

    #[serde(rename = "Address")]
    pub subscription_address: String,

    #[serde(rename = "Message_Binding", default)]
    pub subscription_message_bindings: Vec<String>,
}

/// Content block in poll/inbox messages.
///
/// Note: In TAXII 1.0, content_binding is a simple string, not a ContentBinding object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "Content_Binding")]
    pub content_binding: String,

    #[serde(rename = "Content")]
    pub content: String,

    #[serde(rename = "Timestamp_Label", skip_serializing_if = "Option::is_none")]
    pub timestamp_label: Option<String>,

    #[serde(rename = "Padding", skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
}

/// Poll instance for subscription responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollInstance {
    #[serde(rename = "Protocol_Binding")]
    pub poll_protocol: String,

    #[serde(rename = "Address")]
    pub poll_address: String,

    #[serde(rename = "Message_Binding", default)]
    pub poll_message_bindings: Vec<String>,
}

/// Subscription instance for subscription responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionInstance {
    #[serde(rename = "@subscription_id")]
    pub subscription_id: String,

    #[serde(
        rename = "Delivery_Parameters",
        skip_serializing_if = "Option::is_none"
    )]
    pub delivery_parameters: Option<DeliveryParameters>,

    #[serde(rename = "Poll_Instance", default)]
    pub poll_instances: Vec<PollInstance>,
}

// ============================================================================
// Helper Functions
// ============================================================================

fn default_ns_10() -> String {
    NS_TAXII_10.to_string()
}
